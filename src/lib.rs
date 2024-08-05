pub mod api;
pub mod configuration;
pub mod extractors;
pub mod openapi;
pub mod services;
pub mod state;

use self::openapi::ApiDoc;
use axum::{body::Body, extract::DefaultBodyLimit, routing::get, Router};
use configuration::{reader::ConfigurationReader, *};
use futures::StreamExt;
use migration::{Migrator, MigratorTrait};
use origin::service::Service as CorsService;
use sea_orm::{ActiveModelTrait, ConnectOptions, Database, DbErr, IntoActiveModel};
use serde::{Deserialize, Serialize};
use services::*;
use site::service::Service as SiteService;
use state::State;
use std::{fmt::Debug, path::PathBuf, sync::Arc, time::Duration};
use tokio::{fs, sync::oneshot};
use tower_http::{
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{subscriber::SetGlobalDefaultError, Level};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt::Layer, prelude::*};
use utoipa::{OpenApi, ToSchema};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

/// This struct is a response of server in bad situation
/// That can be INTERNAL SERVER ERROR or BAD REQUEST
/// You can find all information in reason field
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq)]
pub struct Details {
    /// This field will contain error information
    reason: String,
}

/// This struct will be sent to the task
/// to check if origin is allowed for
/// specific subdomain
/// sender here is a oneshot channel to send
/// result back. That is a solution for collisions
pub struct CorsTask {
    pub origin: String,
    pub subdomain: String,
    pub sender: oneshot::Sender<bool>,
}

#[derive(thiserror::Error, Debug)]
pub enum AppCreationError {
    #[error(transparent)]
    EnvReaderError(#[from] <EnvConfigurationReader as ConfigurationReader>::Error),
    #[error(transparent)]
    DatabaseError(#[from] DbErr),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
}

#[tracing::instrument]
pub async fn app() -> Result<(Router, Arc<State>), AppCreationError> {
    tracing::info!("Generating openapi specification for server...");
    let spec = openapi::generate_openapi()?;
    tracing::info!("Openapi specification was successfully generated for server!");

    tracing::info!("Writing generated openapi specification to openapi.json...");
    //* Write openapi spec to file
    tokio::fs::write("./openapi.json", spec).await?;
    tracing::info!("Generated openapi spec was successfully written to openapi.json!");

    //* Read configuration
    let configuration = EnvConfigurationReader::read::<Configuration, PathBuf>(None::<PathBuf>)?;
    tracing::info!("Configuration was successfully read!");

    //* Establish database connection and run migrations
    tracing::info!("Establishing database connection...");
    let mut database_connection_options = ConnectOptions::new(configuration.database_url());
    database_connection_options.sqlx_logging(configuration.sqlx_logging());

    let connection = Database::connect(database_connection_options).await?;
    connection.ping().await?;

    tracing::info!("Database connection was successfully established! Checked with ping command!");

    tracing::info!("Running database migrations....");
    Migrator::up(&connection, None).await?;
    tracing::info!("Successfully finished running necessary database migrations!");

    let state = Arc::new(State::new(connection, configuration.clone()));

    //* This cloned state will be used in spawned task for cors
    let state_for_origins_task = state.clone();

    //* This channel will be used to pass
    //* [`CorsTask`] to spawned task to check allowed origin for subdomain
    let (cors_task_sender, mut cors_task_receiver) = tokio::sync::mpsc::channel::<CorsTask>(100);

    //* This task is responsible for checking if
    //* Origin is allowed for provided subdomain
    //? Why not to check in ['AllowOrigin::async_predicate']?
    //?   dyn futures::Future<Output = Result<std::option::Option<QueryResult>, sea_orm::DbErr>> + std::marker::Send
    //?           cannot be shared between threads safely
    //?   the trait `Sync` is not implemented
    //? And I failed to come up with a solution
    tracing::info!("Spawning task for checking allowed origins [Dynamic Cors Management]...");
    tokio::spawn(async move {
        while let Some(task) = cors_task_receiver.recv().await {
            if let Err(send_error) = task.sender.send(
                CorsService::check_if_origin_is_allowed_for(
                    task.subdomain,
                    task.origin,
                    state_for_origins_task.connection(),
                )
                .await
                .inspect_err(|cause| tracing::warn!(%cause, "Failed to check if origin is allowed!"))
                .unwrap_or(false),
            ) {
                tracing::error!(%send_error, "Failed to send result of checking if origin is allowed!");
            };
        }
    });

    //* This task is responsible for cleanup of obsolete files after uploads
    //* And prevents the server from oom
    //* The task will start with interval defined in CLEAN_OBSOLETE_INTERVAL
    //* If this env was not set the default interval will be 60 seconds
    //* Here the stream is used to avoid unnecessary allocations as there can be 1m of obsolete files
    tracing::info!("Spawning task which is responsible for cleanup...");

    let state_for_file_deletion_task = state.clone();

    tokio::spawn(async move {
        let span = tracing::span!(Level::TRACE, "Cleanup task");
        span.in_scope(|| async move {
            let default_interval = 60;

            let duration = Duration::from_secs(
                state_for_file_deletion_task
                    .configuration()
                    .clean_obsolete_interval()
                    .unwrap_or(default_interval),
            );
            tracing::info!("Cleanup task will run with interval of {} seconds", duration.as_secs());
            let mut interval = tokio::time::interval(duration);

            loop {
                interval.tick().await;
                tracing::debug!("Starting next iteration of cleanup task...");
                if let Ok(mut stream) = SiteService::obsolete(state_for_file_deletion_task.connection())
                    .await
                    .inspect_err(|cause| tracing::warn!(%cause, "Failed to get stream with obsolete files!"))
                {
                    while let Some(Ok(file)) = stream.next().await {
                        tracing::debug!("Removing: {:?}", file.real_path);
                        fs::remove_file(&file.real_path)
                            .await
                            .inspect_err(
                                |cause| tracing::warn!(%cause, "Failed to remove file with path : {}", file.real_path),
                            )
                            .ok();

                        let file_id = file.id;
                        file.into_active_model()
                            .delete(state_for_file_deletion_task.connection())
                            .await
                            .inspect_err(
                                |cause| tracing::warn!(%cause, %file_id, "Failed to remove file from database"),
                            )
                            .ok();
                    }
                }
            }
        })
        .await
    });

    //* According to features of this server we need to check
    //* AllowedOrigin for each request based on x-subdomain header
    //* 1) We retrieve `Origin` header
    //* 2) We retrieve `x-subdomain` header for
    //* 3) We create oneshot channel to receive result of cors check
    //*    back to closure
    //* 4) We send [`CorsTask`] to the spawned task and wait for the result

    tracing::info!("Initializing tower_http::CorsLayer...");

    let cors_layer = CorsLayer::new()
        .allow_methods(AllowMethods::any())
        .allow_headers(AllowHeaders::any())
        .allow_origin(AllowOrigin::async_predicate(move |origin, parts| {
            let retrieved_origin = origin.to_str().unwrap_or_default();
            let retrieved_subdomain = parts.headers.get("x-subdomain").map(|s| s.to_str().unwrap_or_default());

            let (sender, receiver) = tokio::sync::oneshot::channel();
            let task = CorsTask {
                origin: retrieved_origin.to_owned(),
                subdomain: retrieved_subdomain.unwrap_or_default().to_owned(),
                sender,
            };

            async move {
                //? If header was not provided
                //? Allow as it probably management tool
                if task.subdomain.is_empty() {
                    return true;
                }

                if cors_task_sender
                    .send(task)
                    .await
                    .inspect_err(|cause| tracing::warn!(%cause, "Failed to send cors task!"))
                    .is_ok()
                {
                    receiver.await.unwrap_or(false)
                } else {
                    false
                }
            }
        }));

    tracing::info!("Initializing tower_http::trace::TraceLayer...");
    //* Simple tracing which adds uuid to span
    let tracing_layer = TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<Body>| {
        tracing::span!(
            Level::DEBUG,
            "request",
            method = tracing::field::display(request.method()),
            uri = tracing::field::display(request.uri()),
            version = tracing::field::debug(request.version()),
            request_id = tracing::field::display(Uuid::new_v4())
        )
    });

    tracing::info!("Initializing routes for API documentation...");
    //* Generating openapi routes with utoipa
    let openapi = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"));

    let mut app = Router::new()
        .merge(openapi)
        .route("/*path", get(api::site::page::handler::implementation))
        .route("/", get(api::site::page::handler::redirect::implementation))
        .layer(cors_layer)
        .nest("/api", api::router())
        .layer(tracing_layer)
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .with_state(state.clone());

    //* Enable body limit size if required
    if let Some(max_body_limit_size) = &state.configuration().max_body_limit_size() {
        tracing::info!("Setting body limit size to {}", max_body_limit_size);
        app = app.layer(DefaultBodyLimit::max(*max_body_limit_size));
    } else {
        tracing::info!("MAX_BODY_LIMIT_SIZE was not set! Default: 2mb (from axum docs)");
    }

    let upload_folder = state.configuration().upload_folder();

    tracing::info!("Checking if upload folder exists...");
    if !upload_folder.exists() {
        tracing::info!("Creating upload folder...");
        fs::create_dir_all(upload_folder).await?;
    }

    tracing::info!("Done server initialization!");

    Ok((app, state))
}

// Ensure that guard lives to the end of main function
// Otherwise logs will not be flushed
// Just do
//  let _guard = enable_logging().await.expect("Failed to initialize logging");
pub async fn enable_logging() -> Result<WorkerGuard, SetGlobalDefaultError> {
    fs::create_dir("./logs").await.ok();

    let log_file_appender = tracing_appender::rolling::hourly("./logs", "sero.log");
    let (log_file_appender, log_file_guard) = tracing_appender::non_blocking(log_file_appender);

    let stdout_log = tracing_subscriber::fmt::layer()
        .with_test_writer()
        .compact()
        .with_ansi(false); //.pretty() for pretty.
    let subscriber = tracing_subscriber::Registry::default()
        .with(stdout_log)
        .with(Layer::new().with_ansi(false).with_writer(log_file_appender))
        .with(tracing_subscriber::EnvFilter::from_default_env());

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(log_file_guard)
}
