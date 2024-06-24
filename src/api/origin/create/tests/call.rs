#[cfg(test)]
pub mod tests {
    use crate::{
        api::{
            origin::create::{request::AddOriginRequest, response::AddOriginResponse},
            tests::post,
        },
        Details,
    };
    use axum::http::{HeaderName, HeaderValue, StatusCode};
    use axum_test::TestServer as TestClient;
    use std::fmt::Display;

    pub async fn create<T, S, O>(
        client: &TestClient,
        token: T,
        subdomain: S,
        origin: O,
    ) -> Result<AddOriginResponse, (StatusCode, Details)>
    where
        T: AsRef<str> + Display,
        S: AsRef<str> + Display,
        O: Into<String>,
    {
        let origin = AddOriginRequest { origin: origin.into() };

        let response = post(client, "/api/origin", Some(origin))
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
