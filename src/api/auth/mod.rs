use crate::state::State as AppState;
use axum::{routing::post, Router};
use std::sync::Arc;

pub mod login;
pub mod registration;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login::handler::implementation))
        .route("/registration", post(registration::handler::implementation))
}
