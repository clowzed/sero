#[cfg(test)]
mod tests {

    use crate::{
        api::{
            auth::{
                login::{request::LoginRequest, tests::call::test::login},
                registration::{request::RegistrationRequest, tests::call::tests::registration},
            },
            site::{
                disable::tests::call::tests::disable, enable::tests::call::tests::enable,
                page::tests::call::tests::page, upload::tests::call::tests::upload,
            },
        },
        app,
    };
    use axum::http::StatusCode;
    use axum_test::TestServer as TestClient;
    use uuid::Uuid;

    #[tokio::test]
    async fn disable_enable() {
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

        let first_user_registration_response = registration(&client, &first_user_registration_request).await;

        assert!(first_user_registration_response.is_ok());

        let user_login_response = login(&client, &first_user_login_request).await;

        assert!(user_login_response.is_ok());

        let first_user_token = user_login_response.expect("never fail").token;

        let first_random_subdomain = Uuid::new_v4().to_string();
        let second_random_subdomain = Uuid::new_v4().to_string();

        //? Upload correct zip file containing 503.html
        //? Upload correct zip file without 503.html

        let first_subdomain_upload_response = upload(
            &client,
            &first_user_token,
            &first_random_subdomain,
            "./assets/zips/correct-with-503.html.zip",
        )
        .await;

        let second_subdomain_upload_response = upload(
            &client,
            &first_user_token,
            &second_random_subdomain,
            "./assets/zips/correct-without-503.html.zip",
        )
        .await;

        assert!(first_subdomain_upload_response.is_ok());
        assert!(second_subdomain_upload_response.is_ok());

        //? On upload site should be fully available

        let first_subdomain_page_response = page(&client, "/some/index.html", &first_random_subdomain).await;
        let second_subdomain_page_response = page(&client, "/some/index.html", &second_random_subdomain).await;

        assert!(first_subdomain_page_response.status_code().is_success());
        assert!(second_subdomain_page_response.status_code().is_success());

        assert_eq!(first_subdomain_page_response.as_bytes(), "index\n".as_bytes());
        assert_eq!(second_subdomain_page_response.as_bytes(), "index\n".as_bytes());

        //? Call of enable on already enabled should be fine

        let first_subdomain_enable_response = enable(&client, &first_random_subdomain, &first_user_token).await;
        let second_subdomain_enable_response = enable(&client, &second_random_subdomain, &first_user_token).await;

        assert!(first_subdomain_enable_response.is_ok());
        assert!(second_subdomain_enable_response.is_ok());

        let first_subdomain_page_response = page(&client, "/some/index.html", &first_random_subdomain).await;
        let second_subdomain_page_response = page(&client, "/some/index.html", &second_random_subdomain).await;

        assert!(first_subdomain_page_response.status_code().is_success());
        assert!(second_subdomain_page_response.status_code().is_success());

        assert_eq!(first_subdomain_page_response.as_bytes(), "index\n".as_bytes());
        assert_eq!(second_subdomain_page_response.as_bytes(), "index\n".as_bytes());

        //? Now we are going to test disabling

        let first_subdomain_disable_response = disable(&client, &first_random_subdomain, &first_user_token).await;
        let second_subdomain_disable_response = disable(&client, &second_random_subdomain, &first_user_token).await;

        assert!(first_subdomain_disable_response.is_ok());
        assert!(second_subdomain_disable_response.is_ok());

        //? Subdomain with uploaded 503.html should return it and status code 503
        //? Subdomain without uploaded 503.html should return only status code

        let should_be_503_page_response = page(&client, "/some/index.html", &first_random_subdomain).await;
        let should_not_be_503_page_response = page(&client, "/some/index.html", &second_random_subdomain).await;

        assert_eq!(
            should_be_503_page_response.status_code(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            should_not_be_503_page_response.status_code(),
            StatusCode::SERVICE_UNAVAILABLE
        );

        assert_eq!(should_be_503_page_response.as_bytes(), "503\n".as_bytes());
        assert!(should_not_be_503_page_response.as_bytes().is_empty());

        //? Fine. Now enabling back

        let first_subdomain_enable_response = enable(&client, &first_random_subdomain, &first_user_token).await;
        let second_subdomain_enable_response = enable(&client, &second_random_subdomain, &first_user_token).await;

        assert!(first_subdomain_enable_response.is_ok());
        assert!(second_subdomain_enable_response.is_ok());

        let first_subdomain_page_response = page(&client, "/some/index.html", &first_random_subdomain).await;
        let second_subdomain_page_response = page(&client, "/some/index.html", &second_random_subdomain).await;

        assert!(first_subdomain_page_response.status_code().is_success());
        assert!(second_subdomain_page_response.status_code().is_success());

        assert_eq!(first_subdomain_page_response.as_bytes(), "index\n".as_bytes());
        assert_eq!(second_subdomain_page_response.as_bytes(), "index\n".as_bytes());
    }
}
