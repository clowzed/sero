#[cfg(test)]
pub mod tests {
    use crate::{
        api::auth::login::{request::LoginRequest, tests::call::test::login},
        app,
    };
    use axum::http::StatusCode;
    use axum_test::TestServer as TestClient;
    use uuid::Uuid;

    #[tokio::test]
    async fn not_found() {
        dotenvy::from_filename_override(".env.tests").ok();

        let (app, _) = app().await.expect("Failed to initialize application!");
        let client = TestClient::new(app).expect("Failed to run server for testing");

        let first_user_login_request: LoginRequest = LoginRequest {
            login: Uuid::new_v4().into(),
            password: Uuid::new_v4().into(),
        };

        let first_user_login_response = login(&client, &first_user_login_request).await;

        assert!(first_user_login_response.is_err_and(|error| error.0 == StatusCode::NOT_FOUND));
    }
}
