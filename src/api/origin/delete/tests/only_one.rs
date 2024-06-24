#[cfg(test)]
pub mod tests {
    use crate::{
        api::{
            auth::{
                login::{request::LoginRequest, tests::call::test::login},
                registration::{request::RegistrationRequest, tests::call::tests::registration},
            },
            origin::{create::tests::call::tests::create, delete::tests::call::tests::delete, tests::preflight},
            site::{page::tests::call::tests::page, upload::tests::call::tests::upload},
        },
        app,
    };
    use axum_test::TestServer as TestClient;
    use uuid::Uuid;

    #[tokio::test]
    async fn only_one() {
        dotenvy::from_filename_override(".env.tests").ok();

        let (app, _state) = app().await.expect("Failed to initialize application!");
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
        let first_user_login_request = LoginRequest {
            login: first_user_login.into(),
            password: first_user_password.into(),
        };
        let first_user_login_response = login(&client, &first_user_login_request).await;
        assert!(first_user_login_response.is_ok());
        let first_user_token = first_user_login_response.expect("to never fail").token;

        let first_random_subdomain = Uuid::new_v4().to_string();

        let first_correct_zip_path = "./assets/zips/correct-1.zip";

        //* Correct upload
        let first_correct_upload_response = upload(
            &client,
            &first_user_token,
            &first_random_subdomain,
            first_correct_zip_path,
        )
        .await;
        assert!(first_correct_upload_response.is_ok());

        //* Test that page is correct
        let first_user_correct_zip_index_response = page(&client, "/some/index.html", &first_random_subdomain).await;
        assert!(first_user_correct_zip_index_response.status_code().is_success());

        //* Add some origin
        let first_subdomain_first_add_origin_response =
            create(&client, &first_user_token, &first_random_subdomain, "some").await;
        assert!(first_subdomain_first_add_origin_response.is_ok());

        //* Add another origin
        let first_subdomain_second_add_origin_response =
            create(&client, &first_user_token, &first_random_subdomain, "another").await;
        assert!(first_subdomain_second_add_origin_response.is_ok());

        let first_subdomain_first_added_origin = first_subdomain_first_add_origin_response.expect("never fail");

        //* Check origin was added
        let preflight_response = preflight(&client, &first_random_subdomain, "some").await;
        let preflight_response_allowed_origin = preflight_response
            .headers()
            .get(axum::http::header::ACCESS_CONTROL_ALLOW_ORIGIN);

        assert!(preflight_response_allowed_origin.is_some());

        //* Delete
        let first_subdomain_delete_first_origin_response = delete(
            &client,
            &first_user_token,
            &first_random_subdomain,
            first_subdomain_first_added_origin.id,
        )
        .await;
        assert!(first_subdomain_delete_first_origin_response.is_ok());

        //* Check origin again
        let preflight_response = preflight(&client, &first_random_subdomain, "another").await;
        let preflight_response_allowed_origin = preflight_response
            .headers()
            .get(axum::http::header::ACCESS_CONTROL_ALLOW_ORIGIN);

        assert!(preflight_response_allowed_origin.is_some());
    }
}
