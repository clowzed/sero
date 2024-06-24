use super::{error::AddOriginError, request::AddOriginRequest, response::AddOriginResponse};
use crate::{extractors::*, services::origin::service::Service as CorsService, state::State as AppState};
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

/// Adds a new origin to a specified subdomain for dynamic CORS (Cross-Origin Resource Sharing) management.
///
/// This endpoint allows users to add origins that are permitted to access resources
/// on their specified subdomains. The action is authenticated using a JWT, and the subdomain must
/// be owned by the user making the request. This will be checked by the server.
#[utoipa::path(
    post,
    tag = "Origins Management and Dynamic Access Control",
    operation_id = "Create origin",
    path = "/api/origin",
    request_body = AddOriginRequest,
    params(
        ("x-subdomain" = String, 
        Header,
        description = "'x-subdomain' header represents the name of the subdomain on which the action is to be performed."),
    ),
    responses(
        (status = 201, description = "The origin was successfully added.",                                                  body = AddOriginResponse),
        (status = 400, description = "The 'x-subdomain' header is missing or contains invalid characters.",                 body = Details),
        (status = 401, description = "Unauthorized: The JWT in the header is invalid or expired.",                          body = Details),
        (status = 403, description = "Forbidden: The subdomain is owned by another user.",                                  body = Details),
        (status = 404, description = "Not Found: The login or subdomain was not found. See details for more information.",  body = Details),
        (status = 500, description = "Internal Server Error: An error occurred on the server.",                             body = Details),
    ),
    security(("Bearer-JWT" = []))
)]
#[tracing::instrument(skip(state))]
pub async fn implementation(
    State(state): State<Arc<AppState>>,
    SubdomainOwned { user, subdomain }: SubdomainOwned,
    Json(payload): Json<AddOriginRequest>,
) -> Result<impl IntoResponse, AddOriginError> {
    tracing::trace!(
        %payload.origin,
        %subdomain.name,
        %subdomain.id,
        %user.id,
        "Adding origin for subdomain...",
    );

    let added_origin = CorsService::add_origin_for(subdomain.id, payload.origin, state.connection()).await?;
    tracing::trace!(
        ?added_origin,
        %subdomain.name,
        %subdomain.id,
        %user.id,
        "Origin was successfully added to subdomain!",
    );

    let id = added_origin.id;
    let origin = added_origin.value;

    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, format!("/api/origin/{id}"))],
        Json(AddOriginResponse { id, origin }),
    ))
}
