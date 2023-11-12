use std::{fmt::Debug, net::SocketAddr, sync::mpsc};

use axum::{
    extract::DefaultBodyLimit,
    http::StatusCode,
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

async fn app() -> Router {
    let config = config::Config::default();

    let mut opt = ConnectOptions::new(&config.database_url);
    opt.sqlx_logging(true);
    let connection = Database::connect(opt).await.unwrap();
    connection.ping().await.unwrap();

    Migrator::up(&connection, None).await.unwrap();

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

    let cors_layer = CorsLayer::new()
    .allow_methods(AllowMethods::any())
    .allow_headers(AllowHeaders::any())
    .allow_origin(AllowOrigin::predicate(move |origin, parts| {
        let cloned_headers = parts.headers.clone();
        let (tx, rx) = mpsc::channel();

        let cloned_state = cloned_state.clone();
        let cloned_origin = origin
            .to_owned()
            .to_str()
            .map(|s| s.to_string())
            .unwrap_or_default();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                tracing::info!("Starting cors!");
                let subdomain_model_extractor =
                    SubdomainModel::from_headers(&cloned_headers, &cloned_state)
                        .await
                        .map_err(|cause| {
                            tracing::error!(%cause, 
                    "Failed to extract subdomain model from headers for cors!");
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
            });
        });

        rx.recv().unwrap_or(false)
    }));

    let mut app = Router::new()
        .nest("/api", api_router)
        .route("/*path", get(handlers::sites::file))
        .route("/", get(handlers::sites::index_redirect))
        .with_state(state.clone());

    if config.max_body_limit_size.is_some() {
        app = app.layer(DefaultBodyLimit::max(config.max_body_limit_size.unwrap()));
    }

    app
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let config = config::Config::default();
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    axum::Server::bind(&addr)
        .serve(app().await.into_make_service())
        .await
        .unwrap();
}

#[cfg(test)]
mod tests {
    use crate::{handlers::auth::AuthToken, services::auth::AuthCredentials};

    use super::*;
    use axum::{
        body::Body,
        http::{header, HeaderName, HeaderValue, Method, Request, StatusCode},
    };
    use axum_test_helper::TestClient;
    use std::io::Read;
    use tower::ServiceExt; // for `oneshot` and `ready`

    async fn prepare() -> Router {
        dotenv::dotenv().ok();
        //tracing_subscriber::fmt()
        //    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        //    .try_init()
        //    .ok();
        app().await
    }

    #[tokio::test]
    async fn health_check() {
        let app = prepare().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK)
    }

    #[tokio::test]
    async fn registration_check() {
        let app = prepare().await;
        let random_username = uuid::Uuid::new_v4();
        let random_password = uuid::Uuid::new_v4();
        let body = serde_urlencoded::to_string(AuthCredentials {
            username: random_username.to_string(),
            password: random_password.to_string(),
        })
        .unwrap();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/registration")
                    .header(
                        header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(body.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/registration")
                    .header(
                        header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);

        let body = serde_urlencoded::to_string(AuthCredentials {
            username: random_username.to_string(),
            password: String::default(),
        })
        .unwrap();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/registration")
                    .header(
                        header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn login_check() {
        let app = prepare().await;
        let random_username = uuid::Uuid::new_v4();
        let random_password = uuid::Uuid::new_v4();
        let body = serde_urlencoded::to_string(AuthCredentials {
            username: random_username.to_string(),
            password: random_password.to_string(),
        })
        .unwrap();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/registration")
                    .header(
                        header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(body.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/login")
                    .header(
                        header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(body.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/login")
                    .header(
                        header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(body.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

        assert!(serde_json::from_slice::<AuthToken>(&body).is_ok());

        let body = serde_urlencoded::to_string(AuthCredentials {
            username: random_username.to_string(),
            password: String::from("wrong"),
        })
        .unwrap();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/login")
                    .header(
                        header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(body.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn upload_check() {
        let app = prepare().await;
        let random_username = uuid::Uuid::new_v4();
        let random_password = uuid::Uuid::new_v4();
        let first_user = serde_urlencoded::to_string(AuthCredentials {
            username: random_username.to_string(),
            password: random_password.to_string(),
        })
        .unwrap();

        let second_user = serde_urlencoded::to_string(AuthCredentials {
            username: uuid::Uuid::new_v4().to_string(),
            password: uuid::Uuid::new_v4().to_string(),
        })
        .unwrap();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/registration")
                    .header(
                        header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(second_user.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/registration")
                    .header(
                        header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(first_user.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/login")
                    .header(
                        header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(first_user.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

        let token = serde_json::from_slice::<AuthToken>(&body).unwrap().token;

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/login")
                    .header(
                        header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(second_user.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

        let second_token = serde_json::from_slice::<AuthToken>(&body).unwrap().token;

        let client = TestClient::new(app);

        let test_zip = std::path::PathBuf::from("./assets/a.zip");
        assert!(test_zip.exists());

        let mut f = std::fs::File::open(&test_zip).expect("no file found");
        let metadata = std::fs::metadata(&test_zip).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read_exact(&mut buffer).expect("buffer overflow");

        let form = reqwest::multipart::Form::new()
            .part("archive", reqwest::multipart::Part::bytes(buffer.clone()));

        let random_subdoamain = uuid::Uuid::new_v4().to_string();
        let bearer = format!("Bearer {token}");
        let response = client
            .post("/api/upload")
            .multipart(form)
            .header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(&random_subdoamain).unwrap(),
            )
            .header(
                header::AUTHORIZATION,
                HeaderValue::from_str(&bearer).unwrap(),
            )
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let bearer = format!("Bearer {second_token}");
        let form = reqwest::multipart::Form::new()
            .part("archive", reqwest::multipart::Part::bytes(buffer));
        let response = client
            .post("/api/upload")
            .multipart(form)
            .header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(&random_subdoamain).unwrap(),
            )
            .header(
                header::AUTHORIZATION,
                HeaderValue::from_str(&bearer).unwrap(),
            )
            .send()
            .await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn teardown_check() {
        let app = prepare().await;
        let random_username = uuid::Uuid::new_v4();
        let random_password = uuid::Uuid::new_v4();
        let body = serde_urlencoded::to_string(AuthCredentials {
            username: random_username.to_string(),
            password: random_password.to_string(),
        })
        .unwrap();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/registration")
                    .header(
                        header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(body.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/login")
                    .header(
                        header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(body.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

        let token = serde_json::from_slice::<AuthToken>(&body).unwrap().token;

        let client = TestClient::new(app);

        let test_zip = std::path::PathBuf::from("./assets/a.zip");
        assert!(test_zip.exists());

        let mut f = std::fs::File::open(&test_zip).expect("no file found");
        let metadata = std::fs::metadata(&test_zip).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read_exact(&mut buffer).expect("buffer overflow");

        let form = reqwest::multipart::Form::new()
            .part("archive", reqwest::multipart::Part::bytes(buffer));

        let random_subdoamain = uuid::Uuid::new_v4().to_string();
        let bearer = format!("Bearer {token}");
        let response = client
            .post("/api/upload")
            .multipart(form)
            .header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(&random_subdoamain).unwrap(),
            )
            .header(
                header::AUTHORIZATION,
                HeaderValue::from_str(&bearer).unwrap(),
            )
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let response = client
            .get("/a/index.html")
            .header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(&random_subdoamain).unwrap(),
            )
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let response = client
            .post("/api/teardown")
            .header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(&random_subdoamain).unwrap(),
            )
            .header(
                header::AUTHORIZATION,
                HeaderValue::from_str(&bearer).unwrap(),
            )
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let response = client
            .get("/a/index")
            .header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(&random_subdoamain).unwrap(),
            )
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn disable_enable_check() {
        let app = prepare().await;
        let random_username = uuid::Uuid::new_v4();
        let random_password = uuid::Uuid::new_v4();
        let body = serde_urlencoded::to_string(AuthCredentials {
            username: random_username.to_string(),
            password: random_password.to_string(),
        })
        .unwrap();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/registration")
                    .header(
                        header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(body.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/login")
                    .header(
                        header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(body.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

        let token = serde_json::from_slice::<AuthToken>(&body).unwrap().token;

        let client = TestClient::new(app);

        let test_zip = std::path::PathBuf::from("./assets/a.zip");
        assert!(test_zip.exists());

        let mut f = std::fs::File::open(&test_zip).expect("no file found");
        let metadata = std::fs::metadata(&test_zip).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read_exact(&mut buffer).expect("buffer overflow");

        let form = reqwest::multipart::Form::new()
            .part("archive", reqwest::multipart::Part::bytes(buffer));

        let random_subdoamain = uuid::Uuid::new_v4().to_string();
        let bearer = format!("Bearer {token}");
        let response = client
            .post("/api/upload")
            .multipart(form)
            .header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(&random_subdoamain).unwrap(),
            )
            .header(
                header::AUTHORIZATION,
                HeaderValue::from_str(&bearer).unwrap(),
            )
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let response = client
            .get("/a/index.html")
            .header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(&random_subdoamain).unwrap(),
            )
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let response = client
            .post("/api/disable")
            .header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(&random_subdoamain).unwrap(),
            )
            .header(
                header::AUTHORIZATION,
                HeaderValue::from_str(&bearer).unwrap(),
            )
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let response = client
            .get("/a/index")
            .header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(&random_subdoamain).unwrap(),
            )
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

        let response = client
            .post("/api/enable")
            .header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(&random_subdoamain).unwrap(),
            )
            .header(
                header::AUTHORIZATION,
                HeaderValue::from_str(&bearer).unwrap(),
            )
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn download_check() {
        let app = prepare().await;
        let random_username = uuid::Uuid::new_v4();
        let random_password = uuid::Uuid::new_v4();
        let user = serde_urlencoded::to_string(AuthCredentials {
            username: random_username.to_string(),
            password: random_password.to_string(),
        })
        .unwrap();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/registration")
                    .header(
                        header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(user.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/login")
                    .header(
                        header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(user.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

        let token = serde_json::from_slice::<AuthToken>(&body).unwrap().token;

        let client = TestClient::new(app.clone());

        let test_zip = std::path::PathBuf::from("./assets/a.zip");
        assert!(test_zip.exists());

        let mut f = std::fs::File::open(&test_zip).expect("no file found");
        let metadata = std::fs::metadata(&test_zip).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read_exact(&mut buffer).expect("buffer overflow");

        let form = reqwest::multipart::Form::new()
            .part("archive", reqwest::multipart::Part::bytes(buffer.clone()));

        let random_subdoamain = uuid::Uuid::new_v4().to_string();
        let bearer = format!("Bearer {token}");
        let response = client
            .post("/api/upload")
            .multipart(form)
            .header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(&random_subdoamain).unwrap(),
            )
            .header(
                header::AUTHORIZATION,
                HeaderValue::from_str(&bearer).unwrap(),
            )
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let response = client
            .get("/a/index.html")
            .header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(&random_subdoamain).unwrap(),
            )
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let bearer = format!("Bearer {token}");

        let response = app
            .clone()
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/download")
                    .header(
                        HeaderName::from_static("x-subdomain"),
                        HeaderValue::from_str(&random_subdoamain).unwrap(),
                    )
                    .header(
                        header::AUTHORIZATION,
                        HeaderValue::from_str(&bearer).unwrap(),
                    )
                    .body(Body::from(body.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(body, buffer)
    }
}
