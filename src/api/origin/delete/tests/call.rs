#[cfg(test)]
pub mod tests {
    use crate::{api::tests::delete as delete_helper, Details};
    use axum::http::{HeaderName, HeaderValue, StatusCode};
    use axum_test::TestServer as TestClient;
    use std::fmt::Display;

    pub async fn delete<T, S>(client: &TestClient, token: T, subdomain: S, id: i64) -> Result<(), (StatusCode, Details)>
    where
        T: Display,
        S: AsRef<str> + Display,
    {
        let response = delete_helper(client, &format!("/api/origin/{id}"), Option::<()>::None)
            .add_header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(subdomain.as_ref()).expect("Failed to convert subdomain name to header value!"),
            )
            .authorization_bearer(token)
            .await;

        match response.status_code().is_success() {
            true => Ok(()),
            false => Err((response.status_code(), response.json())),
        }
    }
}
