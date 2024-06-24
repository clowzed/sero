use crate::state::State;
use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use std::sync::Arc;

pub mod disable;
pub mod download;
pub mod enable;
pub mod page;
pub mod teardown;
pub mod upload;

pub fn router() -> Router<Arc<State>> {
    Router::new()
        .route("/disable", patch(disable::handler::implementation))
        .route("/enable", patch(enable::handler::implementation))
        .route("/", delete(teardown::handler::implementation))
        .route("/", get(download::handler::implementation))
        .route("/", post(upload::handler::implementation))
}
