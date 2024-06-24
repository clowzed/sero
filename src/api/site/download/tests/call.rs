#[cfg(test)]
pub mod tests {
    use crate::{api::tests::get, Details};
    use axum::http::{HeaderName, HeaderValue, StatusCode};
    use axum_test::TestServer as TestClient;
    use bytes::Bytes;
    use std::fmt::Display;

    pub async fn download<S, T>(client: &TestClient, subdomain: S, token: T) -> Result<Bytes, (StatusCode, Details)>
    where
        S: AsRef<str>,
        T: Display,
    {
        let response = get(client, "/api/site")
            .add_header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(subdomain.as_ref()).expect("Failed to convert subdomain name to header value!"),
            )
            .authorization_bearer(token)
            .await;
        match response.status_code().is_success() {
            true => Ok(response.as_bytes().to_owned()),
            false => Err((response.status_code(), response.json())),
        }
    }
}
