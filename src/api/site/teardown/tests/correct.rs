#[cfg(test)]
mod tests {
    use crate::{
        api::{
            auth::{
                login::{request::LoginRequest, tests::call::test::login},
                registration::{request::RegistrationRequest, tests::call::tests::registration},
            },
            site::{
                page::tests::call::tests::page, teardown::tests::call::tests::teardown,
                upload::tests::call::tests::upload,
            },
        },
        app,
    };
    use axum::http::StatusCode;
    use axum_test::TestServer as TestClient;
    use uuid::Uuid;

    #[tokio::test]
    async fn correct() {
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

        let second_user_login = Uuid::new_v4();
        let second_user_password = Uuid::new_v4();

        let second_user_registration_request = RegistrationRequest {
            login: second_user_login.into(),
            password: second_user_password.into(),
        };

        let second_user_login_request = LoginRequest {
            login: second_user_login.into(),
            password: second_user_password.into(),
        };

        let first_random_subdomain = uuid::Uuid::new_v4().to_string();

        let first_user_registration_response = registration(&client, &first_user_registration_request).await;
        let second_user_registration_response = registration(&client, &second_user_registration_request).await;

        assert!(first_user_registration_response.is_ok());
        assert!(second_user_registration_response.is_ok());

        let user_login_response = login(&client, &first_user_login_request).await;
        let second_user_login_response = login(&client, &second_user_login_request).await;

        assert!(user_login_response.is_ok());
        assert!(second_user_login_response.is_ok());

        let first_user_token = user_login_response.expect("never fail").token;
        let second_user_token = second_user_login_response.expect("never fail").token;

        //? Upload

        let first_zip_path = "./assets/zips/correct-1.zip";
        let index_path = "/some/index.html";

        let first_user_first_zip_upload_response =
            upload(&client, &first_user_token, &first_random_subdomain, first_zip_path).await;
        assert!(first_user_first_zip_upload_response.is_ok());

        //? Check that everything works
        let first_user_first_zip_index_page_response = page(&client, index_path, &first_random_subdomain).await;
        assert!(first_user_first_zip_index_page_response.status_code().is_success());

        //? Teardown now and check that nothing can be found
        let first_user_first_zip_teardown_response =
            teardown(&client, &first_random_subdomain, &first_user_token).await;
        assert_eq!(first_user_first_zip_teardown_response, Ok(()));

        let first_user_first_zip_index_page_response = page(&client, index_path, &first_random_subdomain).await;
        assert_eq!(
            first_user_first_zip_index_page_response.status_code(),
            StatusCode::NOT_FOUND
        );

        //? Upload to same subdomain as it should must free now
        let second_user_first_zip_upload_response =
            upload(&client, &second_user_token, &first_random_subdomain, first_zip_path).await;
        assert!(second_user_first_zip_upload_response.is_ok());

        let second_user_first_zip_index_page_response = page(&client, index_path, &first_random_subdomain).await;
        assert!(second_user_first_zip_index_page_response.status_code().is_success());
    }
}
