#[cfg(test)]
pub mod tests {
    use crate::api::tests::get;
    use axum::http::{HeaderName, HeaderValue};
    use axum_test::{TestResponse, TestServer as TestClient};

    pub async fn page<U, S>(client: &TestClient, url: U, subdomain: S) -> TestResponse
    where
        U: AsRef<str>,
        S: AsRef<str>,
    {
        get(client, url.as_ref())
            .add_header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(subdomain.as_ref()).expect("Failed to convert subdomain name to header value!"),
            )
            .await
    }
}
