use crate::{
    apperror::SeroError,
    extractors::RegistrationGuard,
    services::auth::{AuthCredentials, AuthService, Jwt},
    AppState,
};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Form, Json,
};
use std::sync::Arc;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AuthToken {
    pub token: Jwt,
}

#[tracing::instrument(skip(state))]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Form(credentials): Form<AuthCredentials>,
) -> Response {
    if !credentials.valid() {
        return SeroError::EmptyCredentials.into_response();
    }
    match AuthService::login(
        credentials,
        &state.connection,
        state.config.jwt_secret.as_ref().unwrap(),
    )
    .await
    {
        Ok(Some(token)) => (StatusCode::OK, Json(AuthToken { token })).into_response(),
        Ok(None) => SeroError::Unauthorized.into_response(),
        Err(cause) => SeroError::InternalServerError(Box::new(cause)).into_response(),
    }
}

#[tracing::instrument(skip(state))]
pub async fn registration(
    _: RegistrationGuard,
    State(state): State<Arc<AppState>>,
    Form(credentials): Form<AuthCredentials>,
) -> Response {
    if !credentials.valid() {
        return SeroError::EmptyCredentials.into_response();
    }
    match AuthService::registration(credentials, &state.connection).await {
        Ok(Some(_)) => StatusCode::OK.into_response(),
        Ok(None) => SeroError::UserHasAlreadyBeenRegistered.into_response(),
        Err(cause) => SeroError::InternalServerError(Box::new(cause)).into_response(),
    }
}
