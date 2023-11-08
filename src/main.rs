use std::{fmt::Debug, net::SocketAddr, sync::mpsc};

use axum::{
    extract::DefaultBodyLimit,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use futures::executor::block_on;

use extractors::SubdomainModel;
use services::cors::CorsService;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};

use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database};

mod apperror;
mod config;
mod extractors;
mod handlers;
mod services;

#[derive(Clone, Debug)]
pub struct AppState {
    connection: sea_orm::DatabaseConnection,
    config: config::Config,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let config = config::Config::default();

    let mut opt = ConnectOptions::new(&config.database_url);
    opt.sqlx_logging(true);
    let connection = Database::connect(opt).await.unwrap();
    connection.ping().await.unwrap();

    Migrator::up(&connection, None).await.unwrap();

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

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

    let state = std::sync::Arc::new(AppState {
        connection,
        config: Default::default(),
    });

    let cloned_state = state.clone();

    let mut app = Router::new()
        .nest("/api", api_router)
        .route("/*path", get(handlers::sites::file))
        .route("/", get(handlers::sites::index_redirect))        
        .layer(
            CorsLayer::new()
            .allow_methods(AllowMethods::any())
            .allow_headers(AllowHeaders::any())
                .allow_origin(AllowOrigin::predicate(move |origin, parts| {
                    let cloned_state = cloned_state.clone();
                    let cloned_origin = origin
                        .clone()
                        .to_str()
                        .map(|s| s.to_string())
                        .unwrap_or_default();
                    let cloned_headers = parts.headers.clone();
                    let (tx, rx) = mpsc::channel();

                    std::thread::spawn(
                     move ||  block_on(async move {
                        tracing::info!("Starting cors!");
                        let subdomain_model_extractor =
                        SubdomainModel::from_headers(&cloned_headers, &cloned_state)
                                .await
                                .map_err(|cause| {
                                    tracing::error!(%cause, "Failed to extract subdomain model from headers for cors!");
                                });
                        if subdomain_model_extractor.is_err() {
                            tx.send(false).ok();
                            return;
                        }

                        let res = CorsService::check(
                            subdomain_model_extractor.unwrap().0,
                            &cloned_origin,
                            &cloned_state.connection,
                        )
                        .await
                        .unwrap_or(false);

                        tx.send(res).ok();
                    }
                    ));

                    rx.recv().unwrap_or(false)
                }))
        )
        .with_state(state.clone());

    if config.max_body_limit_size.is_some() {
        app = app.layer(DefaultBodyLimit::max(config.max_body_limit_size.unwrap()));
    }

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
