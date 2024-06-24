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
    async fn bad_credentials() {
        dotenvy::from_filename_override(".env.tests").ok();

        let (app, _) = app().await.expect("Failed to initialize application!");
        let client = TestClient::new(app).expect("Failed to run server for testing");

        let invalid_credentials = LoginRequest {
            login: Uuid::new_v4().into(),
            password: "".into(),
        };

        let invalid_user_login_response = login(&client, &invalid_credentials).await;

        assert!(invalid_user_login_response.is_err_and(|error| error.0 == StatusCode::BAD_REQUEST));
    }
}
