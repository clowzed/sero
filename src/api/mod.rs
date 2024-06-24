use crate::state::State as AppState;
use axum::{http::StatusCode, routing::get, Router};
use std::sync::Arc;

pub mod auth;
pub mod origin;
pub mod site;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/origin", origin::router())
        .nest("/site", site::router())
        .route("/health", get(|| async { StatusCode::OK }))
}

#[cfg(test)]
pub mod tests {
    use axum_test::{TestRequest, TestServer as TestClient};
    use serde::Serialize;

    macro_rules! impl_call {
        ($method:ident) => {
            pub fn $method<T, U>(client: &TestClient, url: U, data: Option<T>) -> TestRequest
            where
                T: Serialize,
                U: AsRef<str>,
            {
                let mut builder = client.$method(url.as_ref());
                if let Some(data) = data {
                    builder = builder.json(&data);
                }
                builder
            }
        };
    }

    impl_call!(post);
    impl_call!(patch);
    impl_call!(delete);

    pub fn get<U>(client: &TestClient, url: U) -> TestRequest
    where
        U: AsRef<str>,
    {
        client.get(url.as_ref())
    }
}
