use std::{fmt::Debug, net::SocketAddr};

use axum::{
    extract::DefaultBodyLimit,
    http::{request::Parts, HeaderName, HeaderValue, Method, StatusCode},
    routing::{get, post},
    Router,
};
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

    let files_router = Router::new()
        .route("/*path", get(handlers::sites::file))
        .route("/", get(handlers::sites::index_redirect))
        .layer(
            CorsLayer::new()
                .allow_methods(AllowMethods::exact(Method::GET))
                .allow_headers(AllowHeaders::list([HeaderName::from_static("X-Subdomain")]))
                .allow_origin(AllowOrigin::predicate(
                    move |origin: &HeaderValue, parts: &Parts| {
                        let origin = origin.to_str().unwrap_or_default();
                        let subdomain_model_future =
                            SubdomainModel::from_headers(&parts.headers, &cloned_state);

                        tokio::runtime::Handle::current().block_on(async {
                            let subdomain_model = subdomain_model_future.await;
                            match subdomain_model {
                                Ok(model) => match CorsService::check(model.0, origin, &cloned_state.connection).await{
                                    Ok(result) => result,
                                    Err(cause) => {
                                        tracing::error!(%cause, "Failed to check origin for cors filtering!");
                                        false
                                    }
                                },
                                Err(cause) => {
                                    tracing::error!(%cause, "Failed to find subdomain model for cors filtering!");
                                    false
                                },
                            }
                        })
                    },
                )),
        );

    let mut app = Router::new()
        .nest("/api", api_router)
        .nest("/", files_router)
        .with_state(state.clone());

    if config.max_body_limit_size.is_some() {
        app = app.layer(DefaultBodyLimit::max(config.max_body_limit_size.unwrap()));
    }

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
