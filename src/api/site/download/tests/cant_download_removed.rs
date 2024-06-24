#[cfg(test)]
mod tests {
    use crate::{
        api::{
            auth::{
                login::{request::LoginRequest, tests::call::test::login},
                registration::{request::RegistrationRequest, tests::call::tests::registration},
            },
            site::{
                download::tests::call::tests::download, teardown::tests::call::tests::teardown,
                upload::tests::call::tests::upload,
            },
        },
        app,
    };
    use axum::http::StatusCode;
    use axum_test::TestServer as TestClient;
    use std::{fs, fs::File, io::Read};
    use uuid::Uuid;

    #[tokio::test]
    async fn cant_download_removed() {
        dotenvy::from_filename_override(".env.tests").ok();

        let (app, _) = app().await.expect("Failed to initialize application!");
        let client = TestClient::new(app).expect("Failed to run server for testing");

        let first_random_subdomain = Uuid::new_v4().to_string();
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

        //* Registration
        let first_user_registration_response = registration(&client, &first_user_registration_request).await;
        assert!(first_user_registration_response.is_ok());

        //* Login
        let first_user_login_response = login(&client, &first_user_login_request).await;
        assert!(first_user_login_response.is_ok());

        let first_user_token = first_user_login_response.expect("never fails").token;

        //* Correct upload and download check

        let first_upload_zip_path = "./assets/zips/correct-1.zip";

        let mut first_f_zip =
            File::open(first_upload_zip_path).unwrap_or_else(|_| panic!("Failed to open {first_upload_zip_path}"));

        //* Writing zip to buffer to compare
        let first_zip_metadata = fs::metadata(first_upload_zip_path).unwrap();
        let mut first_zip_buffer = vec![0; first_zip_metadata.len() as usize];
        first_f_zip.read_exact(&mut first_zip_buffer).unwrap();

        //* Upload
        let first_user_first_zip_upload_response = upload(
            &client,
            &first_user_token,
            &first_random_subdomain,
            first_upload_zip_path,
        )
        .await;
        assert!(first_user_first_zip_upload_response.is_ok());

        //* Download and compare
        let first_user_first_zip_download_response =
            download(&client, &first_random_subdomain, &first_user_token).await;

        assert!(first_user_first_zip_download_response.is_ok());
        assert_eq!(first_user_first_zip_download_response.unwrap(), first_zip_buffer);

        //* Teardown
        let teardown_first_user_first_zip_response =
            teardown(&client, &first_random_subdomain, &first_user_token).await;
        assert!(teardown_first_user_first_zip_response.is_ok());

        //* Actual check that we can't download removed site
        let first_user_first_zip_download_response =
            download(&client, &first_random_subdomain, &first_user_token).await;
        assert!(first_user_first_zip_download_response.is_err_and(|error| error.0 == StatusCode::NOT_FOUND));
    }
}
