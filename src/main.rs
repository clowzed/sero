use std::{fmt::Debug, net::SocketAddr};

use axum::{
    extract::DefaultBodyLimit,
    http::StatusCode,
    routing::{get, post},
    Router,
};
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
        .route("/disable", post(handlers::sites::disable));

    let state = std::sync::Arc::new(AppState {
        connection,
        config: Default::default(),
    });

    let mut app = Router::new()
        .nest("/api", api_router)
        .route("/*path", get(handlers::sites::file))
        .with_state(state.clone());

    if config.max_body_limit_size.is_some() {
        app = app.layer(DefaultBodyLimit::max(config.max_body_limit_size.unwrap()));
    }

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
