#[cfg(test)]
pub mod tests {
    use axum::http::{HeaderName, HeaderValue, StatusCode};
    use axum_test::{
        multipart::{MultipartForm, Part},
        TestServer as TestClient,
    };
    use std::{
        fmt::Display,
        fs::{self, File},
        io::Read as _,
        path::{Path, PathBuf},
    };

    use crate::api::tests::post;

    pub async fn upload<T, S, F>(client: &TestClient, token: T, subdomain: S, filename: F) -> Result<(), StatusCode>
    where
        T: Display,
        S: AsRef<str>,
        F: AsRef<Path>,
    {
        let zip = PathBuf::from(filename.as_ref());

        let mut zip_file = File::open(&zip).expect("Failed to open zip file");
        let metadata = fs::metadata(&zip).expect("Failed to extract metadata from zip file");

        let mut buffer = vec![0; metadata.len() as usize];

        zip_file
            .read_exact(&mut buffer)
            .expect("Failed to read zip bytes into buffer");

        let form = MultipartForm::new().add_part("archive", Part::bytes(buffer));

        let response = post(client, "/api/site", Option::<()>::None)
            .multipart(form)
            .add_header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(subdomain.as_ref()).expect("Failed to convert subdomain name to header value!"),
            )
            .authorization_bearer(token)
            .await;

        match response.status_code().is_success() {
            true => Ok(()),
            false => Err(response.status_code()),
        }
    }
}
