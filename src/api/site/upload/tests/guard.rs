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
    async fn guard() {
        dotenvy::from_filename_override(".env.tests").ok();

        let old_max_sites_per_user = std::env::var("MAX_SITES_PER_USER").unwrap();

        std::env::set_var("MAX_SITES_PER_USER", "2");

        let (app, _) = app().await.expect("Failed to initialize application!");
        std::env::set_var("MAX_SITES_PER_USER", old_max_sites_per_user);

        let client = TestClient::new(app).expect("Failed to run server for testing");

        let first_random_subdomain = Uuid::new_v4().to_string();
        let second_random_subdomain = Uuid::new_v4().to_string();
        let third_random_subdomain = Uuid::new_v4().to_string();

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

        let user_registration_response = registration(&client, &first_user_registration_request).await;
        assert!(user_registration_response.is_ok());

        let user_login_response = login(&client, &first_user_login_request).await;
        assert!(user_login_response.is_ok());

        let first_user_token = user_login_response.expect("never fails").token;

        let first_correct_zip_path = "./assets/zips/correct-1.zip";
        let second_correct_zip_path = "./assets/zips/correct-2.zip";

        let first_correct_upload_response = upload(
            &client,
            &first_user_token,
            &first_random_subdomain,
            first_correct_zip_path,
        )
        .await;
        assert_eq!(first_correct_upload_response, Ok(()));

        let second_correct_upload = upload(
            &client,
            &first_user_token,
            &first_random_subdomain,
            second_correct_zip_path,
        )
        .await;
        assert!(second_correct_upload.is_ok());

        let third_correct_upload_response = upload(
            &client,
            &first_user_token,
            &second_random_subdomain,
            first_correct_zip_path,
        )
        .await;
        assert!(third_correct_upload_response.is_ok());

        let forth_correct_upload_response = upload(
            &client,
            &first_user_token,
            &third_random_subdomain,
            first_correct_zip_path,
        )
        .await;
        assert_eq!(forth_correct_upload_response, Err(StatusCode::FORBIDDEN));
    }
}
