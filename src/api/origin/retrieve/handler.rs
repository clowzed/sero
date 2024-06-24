use super::{error::GetOriginError, response::GetOriginResponse};
use crate::{extractors::*, services::origin::service::Service as CorsService, state::State as AppState};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

/// Get specified origin [by id] for specified subdomain for dynamic CORS (Cross-Origin Resource Sharing) management.
///
/// This endpoint allows users to get specified origin by id that is permitted to access resources
/// on specified subdomain. The action is authenticated using a JWT, and the subdomain must
/// be owned by the user making the request. This will be checked by the server.
#[utoipa::path(
    get,
    tag = "Origins Management and Dynamic Access Control",
    operation_id = "Get origin by id",
    path = "/api/origin/{id}",
    params(
        ("x-subdomain" = String, 
        Header,
        description = "'x-subdomain' header represents the name of the subdomain on which the action is to be performed."),
        ("id" = i64, Path, description = "Id of the origin to retrieve"),
    ),
    responses(
        (status = 200, description = "Origin was successfully retrieved.",                                                            body = GetOriginResponse),
        (status = 400, description = "The 'x-subdomain' header is missing or contains invalid characters.",                           body = Details),
        (status = 401, description = "Unauthorized: The JWT in the header is invalid or expired.",                                    body = Details),
        (status = 403, description = "Forbidden: The subdomain is owned by another user.",                                            body = Details),
        (status = 404, description = "Not Found: The login or subdomain or origin was not found. See details for more information.",  body = Details),
        (status = 500, description = "Internal Server Error: An error occurred on the server.",                                       body = Details),
    ),
    security(("Bearer-JWT" = []))
)]
#[tracing::instrument(skip(state))]
pub async fn implementation(
    State(state): State<Arc<AppState>>,
    SubdomainOwned { user, subdomain }: SubdomainOwned,
    Path(origin_id): Path<i64>,
) -> Result<impl IntoResponse, GetOriginError> {
    tracing::trace!(
        %origin_id,
        %subdomain.name,
        %subdomain.id,
        %user.id,
        "Retrieving origin by id for subdomain...",
    );
    let origin = CorsService::fetch_origin_of(subdomain.id, origin_id, state.connection()).await?;
    tracing::trace!(
        ?origin,
        %subdomain.name,
        %subdomain.id,
        %user.id,
        "Origin was successfully retrieved by id",
    );

    Ok(Json(GetOriginResponse { origin }))
}
