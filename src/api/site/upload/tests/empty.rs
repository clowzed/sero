#[cfg(test)]
mod tests {
    use crate::{
        api::{
            auth::{
                login::{request::LoginRequest, tests::call::test::login},
                registration::{request::RegistrationRequest, tests::call::tests::registration},
            },
            site::upload::tests::call::tests::upload,
        },
        app,
    };
    use axum::http::StatusCode;
    use axum_test::TestServer as TestClient;
    use uuid::Uuid;

    #[tokio::test]
    async fn empty() {
        dotenvy::from_filename_override(".env.tests").ok();

        let (app, _) = app().await.expect("Failed to initialize application!");
        let client = TestClient::new(app).expect("Failed to run server for testing");

        let first_user_login = Uuid::new_v4();
        let first_user_password = Uuid::new_v4();

        let first_user_registration_request = RegistrationRequest {
            login: first_user_login.into(),
            password: first_user_password.into(),
        };

        let first_user_login_request = LoginRequest {
            login: first_user_login.into(),
            password: first_user_password.into(),
        };

        let first_random_subdomain = Uuid::new_v4().to_string();

        let user_registration_response = registration(&client, &first_user_registration_request).await;
        assert!(user_registration_response.is_ok());

        let user_login_response = login(&client, &first_user_login_request).await;
        assert!(user_login_response.is_ok());

        let first_user_token = user_login_response.expect("never fails").token;

        let empty_zip_path = "./assets/zips/empty.zip";

        let empty_upload_response = upload(&client, &first_user_token, &first_random_subdomain, empty_zip_path).await;
        assert_eq!(empty_upload_response, Err(StatusCode::BAD_REQUEST));
    }
}
