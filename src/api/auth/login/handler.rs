use super::{error::LoginError, request::LoginRequest, response::LoginResponse};
use crate::{
    auth::parameters::{JwtGenerationParameters, UserCredentials},
    services::auth::service::Service as AuthService,
    state::State as AppState,
};
use axum::{extract::State, Json};
use std::sync::Arc;
use validator::Validate;

/// Login user and receive JWT token.
///
/// This endpoint allows users to login to sero server. The TTL for token is set by
/// the owner of the server by `JWT_TTL` env.
#[utoipa::path(
    post,
    tag  = "Account management",
    operation_id = "Login",
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "User was successfully authenticated.",         body = LoginResponse),
        (status = 400, description = "Bad request or bad credentials. See details.", body = Details),
        (status = 404, description = "Login was not found.",                         body = Details),
        (status = 500, description = "Some error occurred on the server.",           body = Details),
    ),
)]
#[tracing::instrument(skip(state))]
pub async fn implementation(
    State(state): State<Arc<AppState>>,
    Json(credentials): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, LoginError> {
    //? I am validating here for
    //? custom error response
    //? Safe because of custom Debug impl
    tracing::trace!(?credentials, "Validating provided credentials...");
    credentials.validate()?;
    tracing::trace!(?credentials, "Credentials are valid!");

    let credentials = UserCredentials {
        login: credentials.login,
        password: credentials.password,
    };

    let user = AuthService::login(credentials, state.connection()).await?;
    tracing::trace!(?user, "User was successfully found!");

    let parameters = JwtGenerationParameters {
        secret: state.configuration().jwt_secret(),
        ttl: state.configuration().jwt_ttl_seconds(),
    };

    let token = AuthService::generate_jwt(user.id, parameters)?;
    tracing::trace!("Token was successfully generated!");

    Ok(Json(LoginResponse { token }))
}
