#[cfg(test)]
pub mod tests {
    use crate::{
        api::auth::{
            login::{request::LoginRequest, tests::call::test::login},
            registration::{request::RegistrationRequest, tests::call::tests::registration},
        },
        app,
    };
    use axum_test::TestServer as TestClient;
    use uuid::Uuid;

    #[tokio::test]
    async fn correct() {
        dotenvy::from_filename_override(".env.tests").ok();

        let (app, _) = app().await.expect("Failed to initialize application!");
        let client = TestClient::new(app).expect("Failed to run server for testing");

        let first_user_login = Uuid::new_v4();
        let first_user_password = Uuid::new_v4();

        //* Registration
        let first_user_registration_request = RegistrationRequest {
            login: first_user_login.into(),
            password: first_user_password.into(),
        };

        let first_user_registration_response = registration(&client, &first_user_registration_request).await;
        assert!(first_user_registration_response.is_ok());

        //* Login
        let first_user_login_request: LoginRequest = LoginRequest {
            login: first_user_login.into(),
            password: first_user_password.into(),
        };
        let first_user_login_response = login(&client, &first_user_login_request).await;
        assert!(first_user_login_response.is_ok());

        assert!(!first_user_login_response.unwrap().token.is_empty());
    }
}
