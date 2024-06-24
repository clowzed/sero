#[cfg(test)]
pub mod tests {
    use crate::{
        api::{
            auth::registration::{request::RegistrationRequest, response::RegistrationResponse},
            tests::post,
        },
        Details,
    };
    use axum::http::StatusCode;
    use axum_test::TestServer as TestClient;

    pub async fn registration(
        client: &TestClient,
        credentials: &RegistrationRequest,
    ) -> Result<RegistrationResponse, (StatusCode, Details)> {
        let response = post(client, "/api/auth/registration", Some(credentials)).await;
        match response.status_code().is_success() {
            true => Ok(response.json::<RegistrationResponse>()),
            false => Err((response.status_code(), response.json())),
        }
    }
}
