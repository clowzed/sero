use crate::state::State as AppState;
use axum::{
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use utoipa::ToSchema;

pub mod create;
pub mod delete;
pub mod list;
pub mod purge;
pub mod retrieve;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create::handler::implementation))
        .route("/", get(list::handler::implementation))
        .route("/", delete(purge::handler::implementation))
        .route("/:id", delete(delete::handler::implementation))
        .route("/:id", get(retrieve::handler::implementation))
}

// We need this as utoipa
// currently does not support types
// from external crates
#[derive(ToSchema)]
#[schema(as = OriginModel)]
pub struct OriginModelSchema {
    pub id: i64,
    pub subdomain_id: i64,
    pub value: String,
}

#[cfg(test)]
pub mod tests {
    use axum::http::{HeaderName, HeaderValue};
    use axum_test::{TestResponse, TestServer};

    //? That is a common call of preflight
    //? for testing in all origins handlers
    pub async fn preflight<S, O>(app: &TestServer, subdomain: S, origin: O) -> TestResponse
    where
        S: AsRef<str>,
        O: AsRef<str>,
    {
        app.method(axum::http::Method::OPTIONS, "/")
            .add_header(
                HeaderName::from_static("x-subdomain"),
                HeaderValue::from_str(subdomain.as_ref()).expect("Failed to convert subdomain name to header value!"),
            )
            .add_header(
                axum::http::header::ORIGIN,
                HeaderValue::from_str(origin.as_ref()).expect("Failed to convert origin to header value!"),
            )
            .add_header(
                axum::http::header::ACCESS_CONTROL_REQUEST_METHOD,
                HeaderValue::from_static("GET"),
            )
            .add_header(
                axum::http::header::ACCESS_CONTROL_REQUEST_HEADERS,
                HeaderValue::from_static("x-subdomain"),
            )
            .await
    }
}
