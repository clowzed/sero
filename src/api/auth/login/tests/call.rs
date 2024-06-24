#[cfg(test)]
pub mod test {
    use crate::{
        api::{
            auth::login::{request::LoginRequest, response::LoginResponse},
            tests::post,
        },
        Details,
    };
    use axum::http::StatusCode;
    use axum_test::TestServer as TestClient;

    pub async fn login(
        client: &TestClient,
        credentials: &LoginRequest,
    ) -> Result<LoginResponse, (StatusCode, Details)> {
        let response = post(client, "/api/auth/login", Some(credentials)).await;

        match response.status_code().is_success() {
            true => Ok(response.json::<LoginResponse>()),
            false => Err((response.status_code(), response.json())),
        }
    }
}
