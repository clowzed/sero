#[cfg(test)]
mod tests {
    use tracing_subscriber::prelude::*;

    use axum::{
        body::Body,
        http::{header, HeaderName, HeaderValue, Method, Request, StatusCode},
        Router,
    };
    use axum_test_helper::TestClient;
    use reqwest::multipart::{Form, Part};
    use sero::{
        app,
        handlers::{auth::AuthToken, cors::OriginForm},
        services::auth::AuthCredentials,
    };
    use std::io::Read;
    use tower::ServiceExt;

    type TestResponse =
        hyper::Response<http_body::combinators::UnsyncBoxBody<bytes::Bytes, axum::Error>>;

    async fn registration(app: &Router, credentials: &AuthCredentials) -> TestResponse {
        let body = serde_urlencoded::to_string(credentials).unwrap();
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/registration")
            .header(
                header::CONTENT_TYPE,
                mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
            )
            .body(Body::from(body))
            .unwrap();

        app.clone().oneshot(request).await.unwrap()
    }

    async fn login(app: &Router, credentials: &AuthCredentials) -> TestResponse {
        let body = serde_urlencoded::to_string(credentials).unwrap();
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/login")
            .header(
                header::CONTENT_TYPE,
                mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
            )
            .body(Body::from(body))
            .unwrap();

        app.clone().oneshot(request).await.unwrap()
    }

    async fn teardown(
        app: &Router,
        token: &str,
        subdomain: &str,
    ) -> axum_test_helper::TestResponse {
        let client = TestClient::new(app.clone());
        let bearer = format!("Bearer {token}");

        client
            .post("/api/teardown")
            .header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(subdomain).unwrap(),
            )
            .header(
                header::AUTHORIZATION,
                HeaderValue::from_str(&bearer).unwrap(),
            )
            .send()
            .await
    }

    async fn enable(app: &Router, token: &str, subdomain: &str) -> axum_test_helper::TestResponse {
        let client = TestClient::new(app.clone());
        let bearer = format!("Bearer {token}");

        client
            .post("/api/enable")
            .header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(subdomain).unwrap(),
            )
            .header(
                header::AUTHORIZATION,
                HeaderValue::from_str(&bearer).unwrap(),
            )
            .send()
            .await
    }

    async fn disable(app: &Router, token: &str, subdomain: &str) -> axum_test_helper::TestResponse {
        let client = TestClient::new(app.clone());
        let bearer = format!("Bearer {token}");

        client
            .post("/api/disable")
            .header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(subdomain).unwrap(),
            )
            .header(
                header::AUTHORIZATION,
                HeaderValue::from_str(&bearer).unwrap(),
            )
            .send()
            .await
    }

    async fn download(
        app: &Router,
        token: &str,
        subdomain: &str,
    ) -> axum_test_helper::TestResponse {
        let client = TestClient::new(app.clone());
        let bearer = format!("Bearer {token}");

        client
            .post("/api/download")
            .header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(subdomain).unwrap(),
            )
            .header(
                header::AUTHORIZATION,
                HeaderValue::from_str(&bearer).unwrap(),
            )
            .send()
            .await
    }

    async fn add_origin(
        app: &Router,
        token: &str,
        subdomain: &str,
        origin: &str,
    ) -> axum_test_helper::TestResponse {
        let client = TestClient::new(app.clone());
        let bearer = format!("Bearer {token}");
        let origin = OriginForm {
            origin: origin.to_owned(),
        };
        client
            .post("/api/cors/add")
            .header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(subdomain).unwrap(),
            )
            .header(
                header::AUTHORIZATION,
                HeaderValue::from_str(&bearer).unwrap(),
            )
            .header(
                header::CONTENT_TYPE,
                mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
            )
            .body(Body::from(serde_urlencoded::to_string(origin).unwrap()))
            .send()
            .await
    }

    async fn clear_origins(
        app: &Router,
        token: &str,
        subdomain: &str,
    ) -> axum_test_helper::TestResponse {
        let client = TestClient::new(app.clone());
        let bearer = format!("Bearer {token}");
        client
            .post("/api/cors/clear")
            .header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(subdomain).unwrap(),
            )
            .header(
                header::AUTHORIZATION,
                HeaderValue::from_str(&bearer).unwrap(),
            )
            .send()
            .await
    }

    async fn upload(app: &Router, subdomain: &str, token: &str) -> axum_test_helper::TestResponse {
        let zip = std::path::PathBuf::from("./assets/a.zip");
        assert!(zip.exists());

        let client = TestClient::new(app.clone());

        let mut f = std::fs::File::open(&zip).unwrap();
        let metadata = std::fs::metadata(&zip).unwrap();
        let mut buffer = vec![0; metadata.len() as usize];

        f.read_exact(&mut buffer).unwrap();

        let form = Form::new().part("archive", Part::bytes(buffer.clone()));

        let bearer = format!("Bearer {token}");

        client
            .post("/api/upload")
            .multipart(form)
            .header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(subdomain).unwrap(),
            )
            .header(
                header::AUTHORIZATION,
                HeaderValue::from_str(&bearer).unwrap(),
            )
            .send()
            .await
    }

    async fn get_page(app: &Router, page: &str, subdomain: &str) -> axum_test_helper::TestResponse {
        let client = TestClient::new(app.clone());

        client
            .get(page)
            .header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(subdomain).unwrap(),
            )
            .send()
            .await
    }

    async fn prepare() -> Router {
        dotenv::dotenv().ok();

        let stdout_log = tracing_subscriber::fmt::layer().pretty();
        let subscriber = tracing_subscriber::Registry::default()
            .with(stdout_log)
            .with(tracing_subscriber::EnvFilter::from_default_env());

        tracing::subscriber::set_global_default(subscriber).ok();
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

        let first_user = AuthCredentials::random();
        let unvalid_credentials = AuthCredentials::random_unvalid();

        let first_user_registration_response = registration(&app, &first_user).await;
        assert!(first_user_registration_response.status().is_success());

        let first_user_second_attempt_response = registration(&app, &first_user).await;
        assert_eq!(
            first_user_second_attempt_response.status(),
            (StatusCode::CONFLICT)
        );

        let invalid_credentials_attempt_response = registration(&app, &unvalid_credentials).await;
        assert_eq!(
            invalid_credentials_attempt_response.status(),
            (StatusCode::BAD_REQUEST)
        );
    }

    #[tokio::test]
    async fn login_check() {
        let app = prepare().await;

        let first_user = AuthCredentials::random();
        let second_user = AuthCredentials::random();
        let invalid_credentials = AuthCredentials::random_unvalid();

        let first_user_register_response = registration(&app, &first_user).await;
        assert!(first_user_register_response.status().is_success());

        let first_user_login_response = login(&app, &first_user).await;
        assert!(first_user_login_response.status().is_success());

        let body = hyper::body::to_bytes(first_user_login_response.into_body())
            .await
            .unwrap();
        let token = serde_json::from_slice::<AuthToken>(&body).unwrap().token;

        assert!(!token.is_empty());

        let second_user_login_response = login(&app, &second_user).await;
        assert_eq!(
            second_user_login_response.status(),
            StatusCode::UNAUTHORIZED
        );

        let invalid_user_login_response = login(&app, &invalid_credentials).await;
        assert_eq!(
            invalid_user_login_response.status(),
            StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    async fn upload_check() {
        let app = prepare().await;

        let first_user = AuthCredentials::random();
        let second_user = AuthCredentials::random();

        let random_subdomain = uuid::Uuid::new_v4().to_string();

        assert!(registration(&app, &first_user).await.status().is_success());
        assert!(registration(&app, &second_user).await.status().is_success());

        let first_user_login_response = login(&app, &first_user).await;
        let second_user_login_response = login(&app, &second_user).await;

        assert!(first_user_login_response.status().is_success());
        assert!(second_user_login_response.status().is_success());

        let first_user_token = serde_json::from_slice::<AuthToken>(
            &hyper::body::to_bytes(first_user_login_response.into_body())
                .await
                .unwrap(),
        )
        .unwrap()
        .token;

        let second_user_token = serde_json::from_slice::<AuthToken>(
            &hyper::body::to_bytes(second_user_login_response.into_body())
                .await
                .unwrap(),
        )
        .unwrap()
        .token;

        assert_eq!(
            upload(&app, &random_subdomain, &first_user_token)
                .await
                .status(),
            StatusCode::OK
        );
        assert_eq!(
            upload(&app, &random_subdomain, &second_user_token)
                .await
                .status(),
            StatusCode::FORBIDDEN
        );

        let page_response = get_page(&app, "/a/index.html", &random_subdomain).await;
        assert_eq!(page_response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn teardown_check() {
        let app = prepare().await;

        let first_user = AuthCredentials::random();
        let second_user = AuthCredentials::random();

        let random_subdomain = uuid::Uuid::new_v4().to_string();

        assert!(registration(&app, &first_user).await.status().is_success());
        assert!(registration(&app, &second_user).await.status().is_success());

        let first_user_login_response = login(&app, &first_user).await;
        let second_user_login_response = login(&app, &second_user).await;

        assert!(first_user_login_response.status().is_success());
        assert!(second_user_login_response.status().is_success());

        let first_user_token = serde_json::from_slice::<AuthToken>(
            &hyper::body::to_bytes(first_user_login_response.into_body())
                .await
                .unwrap(),
        )
        .unwrap()
        .token;
        let second_user_token = serde_json::from_slice::<AuthToken>(
            &hyper::body::to_bytes(second_user_login_response.into_body())
                .await
                .unwrap(),
        )
        .unwrap()
        .token;

        assert!(upload(&app, &random_subdomain, &first_user_token)
            .await
            .status()
            .is_success());
        assert_eq!(
            upload(&app, &random_subdomain, &second_user_token)
                .await
                .status(),
            StatusCode::FORBIDDEN
        );

        let page_response = get_page(&app, "/a/index.html", &random_subdomain).await;
        assert_eq!(page_response.status(), StatusCode::OK);

        let teardown_response = teardown(&app, &first_user_token, &random_subdomain).await;
        assert!(teardown_response.status().is_success());

        let page_response = get_page(&app, "/a/index.html", &random_subdomain).await;
        assert_eq!(page_response.status(), StatusCode::NOT_FOUND);

        assert!(upload(&app, &random_subdomain, &second_user_token)
            .await
            .status()
            .is_success());

        let page_response = get_page(&app, "/a/index.html", &random_subdomain).await;
        assert_eq!(page_response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn disable_enable_check() {
        let app = prepare().await;

        let first_user = AuthCredentials::random();

        let random_subdomain = uuid::Uuid::new_v4().to_string();

        assert!(registration(&app, &first_user).await.status().is_success());

        let first_user_login_response = login(&app, &first_user).await;

        assert!(first_user_login_response.status().is_success());

        let first_user_token = serde_json::from_slice::<AuthToken>(
            &hyper::body::to_bytes(first_user_login_response.into_body())
                .await
                .unwrap(),
        )
        .unwrap()
        .token;

        assert!(upload(&app, &random_subdomain, &first_user_token)
            .await
            .status()
            .is_success());

        assert!(get_page(&app, "/a/index.html", &random_subdomain)
            .await
            .status()
            .is_success());
        assert!(enable(&app, &first_user_token, &random_subdomain)
            .await
            .status()
            .is_success());
        assert!(get_page(&app, "/a/index.html", &random_subdomain)
            .await
            .status()
            .is_success());

        assert!(disable(&app, &first_user_token, &random_subdomain)
            .await
            .status()
            .is_success());
        assert_eq!(
            get_page(&app, "/a/index.html", &random_subdomain)
                .await
                .status(),
            StatusCode::SERVICE_UNAVAILABLE
        );

        assert!(enable(&app, &first_user_token, &random_subdomain)
            .await
            .status()
            .is_success());
        assert!(get_page(&app, "/a/index.html", &random_subdomain)
            .await
            .status()
            .is_success());
    }

    #[tokio::test]
    async fn download_check() {
        let app = prepare().await;

        let first_user = AuthCredentials::random();

        let random_subdomain = uuid::Uuid::new_v4().to_string();

        assert!(registration(&app, &first_user).await.status().is_success());

        let first_user_login_response = login(&app, &first_user).await;

        assert!(first_user_login_response.status().is_success());

        let first_user_token = serde_json::from_slice::<AuthToken>(
            &hyper::body::to_bytes(first_user_login_response.into_body())
                .await
                .unwrap(),
        )
        .unwrap()
        .token;

        assert!(upload(&app, &random_subdomain, &first_user_token)
            .await
            .status()
            .is_success());

        assert!(get_page(&app, "/a/index.html", &random_subdomain)
            .await
            .status()
            .is_success());

        let zip = std::path::PathBuf::from("./assets/a.zip");
        assert!(zip.exists());

        let mut f = std::fs::File::open(&zip).unwrap();
        let metadata = std::fs::metadata(&zip).unwrap();
        let mut buffer = vec![0; metadata.len() as usize];

        f.read_exact(&mut buffer).unwrap();

        let download_response = download(&app, &first_user_token, &random_subdomain).await;
        assert!(download_response.status().is_success());
        let body = download_response.bytes().await;
        assert_eq!(body, buffer)
    }

    #[tokio::test]
    async fn cors_check() {
        let app = prepare().await;

        let first_user = AuthCredentials::random();

        let random_subdomain = uuid::Uuid::new_v4().to_string();

        assert!(registration(&app, &first_user).await.status().is_success());

        let first_user_login_response = login(&app, &first_user).await;

        assert!(first_user_login_response.status().is_success());

        let first_user_token = serde_json::from_slice::<AuthToken>(
            &hyper::body::to_bytes(first_user_login_response.into_body())
                .await
                .unwrap(),
        )
        .unwrap()
        .token;

        assert!(upload(&app, &random_subdomain, &first_user_token)
            .await
            .status()
            .is_success());

        assert!(get_page(&app, "/a/index.html", &random_subdomain)
            .await
            .status()
            .is_success());

        assert!(clear_origins(&app, &first_user_token, &random_subdomain)
            .await
            .status()
            .is_success());

        assert!(clear_origins(&app, &first_user_token, &random_subdomain)
            .await
            .status()
            .is_success());

        assert!(
            add_origin(&app, &first_user_token, &random_subdomain, "some")
                .await
                .status()
                .is_success()
        );

        let preflight_response = preflight(&app, &random_subdomain, "some").await;
        let preflight_response_allowed_origin = preflight_response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN);
        assert!(preflight_response_allowed_origin.is_some());

        let bad_preflight_response = preflight(&app, &random_subdomain, "wrong").await;
        let bad_preflight_response_allowed_origin = bad_preflight_response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN);
        assert!(bad_preflight_response_allowed_origin.is_none());

        let wildcrd_add_origin_response =
            add_origin(&app, &first_user_token, &random_subdomain, "*").await;

        assert!(wildcrd_add_origin_response.status().is_success());

        let random_origin = uuid::Uuid::new_v4().to_string();

        let preflight_response = preflight(&app, &random_subdomain, &random_origin).await;
        let preflight_response_allowed_origin = preflight_response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN);

        assert!(preflight_response_allowed_origin.is_some());
        assert!(preflight_response_allowed_origin.unwrap() == &random_origin);
    }

    async fn preflight(app: &Router, random_subdomain: &str, origin: &str) -> TestResponse {
        app.clone()
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/")
                    .header(
                        HeaderName::from_static("x-subdomain"),
                        HeaderValue::from_str(random_subdomain).unwrap(),
                    )
                    .header(header::ORIGIN, HeaderValue::from_str(origin).unwrap())
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                    .header(header::ACCESS_CONTROL_REQUEST_HEADERS, "X-Subdomain")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    }
}
