#[cfg(test)]
mod tests {
    use crate::{
        api::{
            auth::{
                login::{request::LoginRequest, tests::call::test::login},
                registration::{request::RegistrationRequest, tests::call::tests::registration},
            },
            site::{download::tests::call::tests::download, upload::tests::call::tests::upload},
        },
        app,
    };
    use axum_test::TestServer as TestClient;
    use std::{fs, fs::File, io::Read};
    use uuid::Uuid;

    #[tokio::test]
    async fn correct_many() {
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

        //? Just register and log in

        let first_random_subdomain = Uuid::new_v4().to_string();
        let second_random_subdomain = Uuid::new_v4().to_string();

        let first_user_registration_response = registration(&client, &first_user_registration_request).await;
        let second_user_registration_response = registration(&client, &second_user_registration_request).await;

        assert!(first_user_registration_response.is_ok());
        assert!(second_user_registration_response.is_ok());

        let first_user_login_response = login(&client, &first_user_login_request).await;
        let second_user_login_response = login(&client, &second_user_login_request).await;

        assert!(first_user_login_response.is_ok());
        assert!(second_user_login_response.is_ok());

        let first_user_token = first_user_login_response.expect("never fails").token;
        let second_user_token = second_user_login_response.expect("never fails").token;

        //? Preparation for uploading and cmp

        let first_upload_zip_path = "./assets/zips/correct-1.zip";
        let second_upload_zip_path = "./assets/zips/correct-2.zip";

        let mut first_f_zip =
            File::open(first_upload_zip_path).unwrap_or_else(|_| panic!("Couldn't open {first_upload_zip_path}"));
        let mut second_f_zip =
            File::open(second_upload_zip_path).unwrap_or_else(|_| panic!("Couldn't open {second_upload_zip_path}"));

        let first_zip_metadata = fs::metadata(first_upload_zip_path).unwrap();
        let second_zip_metadata = fs::metadata(second_upload_zip_path).unwrap();

        let mut first_zip_buffer = vec![0; first_zip_metadata.len() as usize];
        let mut second_zip_buffer = vec![0; second_zip_metadata.len() as usize];

        first_f_zip.read_exact(&mut first_zip_buffer).unwrap();
        second_f_zip.read_exact(&mut second_zip_buffer).unwrap();

        //? Uploading first time

        let first_user_first_zip_upload_response = upload(
            &client,
            &first_user_token,
            &first_random_subdomain,
            first_upload_zip_path,
        )
        .await;
        let second_user_second_zip_upload_response = upload(
            &client,
            &second_user_token,
            &second_random_subdomain,
            second_upload_zip_path,
        )
        .await;

        assert!(first_user_first_zip_upload_response.is_ok());
        assert!(second_user_second_zip_upload_response.is_ok());

        //? Downloading them back

        let first_user_first_zip_download_response =
            download(&client, &first_random_subdomain, &first_user_token).await;
        let second_user_second_zip_download_response =
            download(&client, &second_random_subdomain, &second_user_token).await;

        assert!(first_user_first_zip_download_response.is_ok());
        assert!(second_user_second_zip_download_response.is_ok());

        assert_eq!(first_user_first_zip_download_response.unwrap(), first_zip_buffer);
        assert_eq!(second_user_second_zip_download_response.unwrap(), second_zip_buffer);
    }
}
