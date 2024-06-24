use super::{error::RegistrationError, request::RegistrationRequest, response::RegistrationResponse};
use crate::{
    auth::parameters::UserCredentials, extractors::*, services::auth::service::Service as AuthService,
    state::State as AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sea_orm::TransactionTrait;
use std::sync::Arc;
use validator::Validate;

/// Register new user for sero server.
///
/// This endpoint creates new user for sero server. The amount of users is checked
/// by [RegistrationGuard]. The amount of allowed users is determined by `MAX_USERS` env.
#[utoipa::path(
    post,
    tag  = "Account management",
    operation_id = "Registration",
    path = "/api/auth/registration",
    request_body = RegistrationRequest,
    responses(
        (status = 200, description = "User was successfully registered.",            body = RegistrationResponse),
        (status = 400, description = "Bad request or bad credentials. See details.", body = Details),
        (status = 409, description = "Login has already been registered.",           body = Details),
        (status = 500, description = "Some error occurred on the server.",           body = Details),
    ),
)]
#[tracing::instrument(skip(state))]
pub async fn implementation(
    _: RegistrationGuard,
    State(state): State<Arc<AppState>>,
    Json(credentials): Json<RegistrationRequest>,
) -> Result<impl IntoResponse, RegistrationError> {
    //? I am validating here for
    //? custom error response
    //? Safe because of custom Debug impl
    tracing::trace!(?credentials, "Validating provided credentials...");
    credentials.validate()?;
    tracing::trace!(?credentials, "Credentials are valid!");
    let transaction = state.connection().begin().await?;

    let credentials = UserCredentials {
        login: credentials.login,
        password: credentials.password,
    };

    let user_id = AuthService::registration(credentials, &transaction).await?.id;
    tracing::trace!(%user_id, "User was created! Committing changes...");

    transaction.commit().await?;
    tracing::trace!("Successfully committed!");

    Ok((
        StatusCode::CREATED,
        // Leave this one in case of api additions
        // to match REST
        //[(header::LOCATION, format!("/api/user/{user_id}"))],
        Json(RegistrationResponse { id: user_id }),
    ))
}
