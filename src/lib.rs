use std::time::Duration;

use axum::{
    extract::DefaultBodyLimit,
    http::{HeaderName, HeaderValue},
    routing::{get, post},
    Router,
};
use extractors::SubdomainModel;
use hyper::{HeaderMap, StatusCode};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database};
use services::cors::CorsService;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};
use tracing_subscriber::prelude::*;

pub mod apperror;
pub mod config;
pub mod extractors;
pub mod handlers;
pub mod services;

#[derive(Clone, Debug)]
pub struct AppState {
    pub connection: sea_orm::DatabaseConnection,
    pub config: config::Config,
}

pub struct CorsTask {
    origin: String,
    subdomain: String,
    sender: oneshot::Sender<bool>,
}

pub async fn initialize_database(
    config: &config::Config,
    migrations: bool,
) -> sea_orm::DatabaseConnection {
    let mut opt = ConnectOptions::new(&config.database_url);
    opt.sqlx_logging(true);

    let connection = Database::connect(opt).await.unwrap();

    connection.ping().await.unwrap();

    if migrations {
        Migrator::up(&connection, None).await.unwrap();
    }
    connection
}

pub async fn app() -> Router {
    let config = config::Config::default();

    let connection = initialize_database(&config, true).await;
    let state = std::sync::Arc::new(AppState {
        connection,
        config: Default::default(),
    });

    let (cors_task_sender, cors_task_receiver) = crossfire::mpsc::unbounded_future::<CorsTask>();

    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async move {
            let config = config::Config::default();

            let connection = initialize_database(&config, false).await;

            let state = std::sync::Arc::new(AppState { connection, config });
            loop {
                if let Ok(task) = cors_task_receiver.recv().await {
                    let mut map = HeaderMap::new();

                    map.insert(
                        HeaderName::from_static("x-subdomain"),
                        HeaderValue::from_str(&task.subdomain).unwrap(),
                    );

                    let subdomain_model_extractor = SubdomainModel::from_headers(&map, &state)
                        .await
                        .map_err(|cause| {
                            tracing::error!(%cause,
                    "Failed to extract subdomain model from headers for cors!");
                        });
                    match subdomain_model_extractor.is_err() {
                        true => {
                            task.sender.send(false).unwrap();
                        }
                        false => {
                            let res = CorsService::check(
                                subdomain_model_extractor.unwrap().0,
                                &task.origin,
                                &state.connection,
                            )
                            .await
                            .unwrap();
                            task.sender.send(res).unwrap();
                        }
                    }
                }
            }
        })
    });

    let api_router = Router::new()
        .route("/health", get(|| async { StatusCode::OK }))
        .route("/login", post(handlers::auth::login))
        .route("/registration", post(handlers::auth::registration))
        .route("/upload", post(handlers::sites::upload))
        .route("/teardown", post(handlers::sites::teardown))
        .route("/download", post(handlers::sites::download))
        .route("/enable", post(handlers::sites::enable))
        .route("/disable", post(handlers::sites::disable))
        .route("/cors/add", post(handlers::cors::add_origin))
        .route("/cors/clear", post(handlers::cors::clear_all));

    let cors_layer = CorsLayer::new()
        .allow_methods(AllowMethods::any())
        .allow_headers(AllowHeaders::any())
        .allow_origin(AllowOrigin::predicate(move |origin, parts| {
            let (tx, rx) = oneshot::channel();

            let cloned_origin = origin.to_owned().to_str().unwrap_or_default().to_string();
            let cloned_subdomain: String = parts
                .headers
                .get("X-Subdomain")
                .map(|header| header.to_str().unwrap_or_default().to_string())
                .unwrap();

            cors_task_sender
                .try_send(CorsTask {
                    origin: cloned_origin,
                    subdomain: cloned_subdomain,
                    sender: tx,
                })
                .ok();

            rx.recv_timeout(Duration::from_secs(3)).unwrap_or(false)
        }));

    let mut app = Router::new()
        .nest("/api", api_router)
        .route("/*path", get(handlers::sites::file))
        .route("/", get(handlers::sites::index_redirect))
        .layer(cors_layer)
        .with_state(state.clone());

    if config.max_body_limit_size.is_some() {
        app = app.layer(DefaultBodyLimit::max(config.max_body_limit_size.unwrap()));
    }

    app
}

pub async fn prepare() -> Router {
    dotenv::dotenv().ok();

    let stdout_log = tracing_subscriber::fmt::layer().pretty();
    let subscriber = tracing_subscriber::Registry::default()
        .with(stdout_log)
        .with(tracing_subscriber::EnvFilter::from_default_env());

    tracing::subscriber::set_global_default(subscriber).ok();
    app().await
}
