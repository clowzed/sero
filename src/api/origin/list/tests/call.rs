#[cfg(test)]
pub mod tests {
    use crate::{
        api::{origin::list::response::ListOriginsResponse, tests::get},
        Details,
    };
    use axum::http::{HeaderName, HeaderValue, StatusCode};
    use axum_test::TestServer as TestClient;
    use std::fmt::Display;

    pub async fn list<T, S>(
        client: &TestClient,
        token: T,
        subdomain: S,
    ) -> Result<ListOriginsResponse, (StatusCode, Details)>
    where
        T: Display,
        S: AsRef<str> + Display,
    {
        let response = get(client, "/api/origin")
            .add_header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(subdomain.as_ref()).expect("Failed to convert subdomain name to header value!"),
            )
            .authorization_bearer(token)
            .await;

        match response.status_code().is_success() {
            true => Ok(response.json()),
            false => Err((response.status_code(), response.json())),
        }
    }
}
