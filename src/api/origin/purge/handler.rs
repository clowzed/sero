use super::error::DeleteOriginsError;
use crate::{extractors::*, services::origin::service::Service as CorsService, state::State as AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

/// Delete all origins for specified subdomain for dynamic CORS (Cross-Origin Resource Sharing) management.
///
/// This endpoint allows users to delete all origins that are permitted to access resources
/// on their specified subdomains. The action is authenticated using a JWT, and the subdomain must
/// be owned by the user making the request. This will be checked by the server.
#[utoipa::path(
    delete,
    tag = "Origins Management and Dynamic Access Control",
    operation_id = "Delete all origins",
    path = "/api/origin",
    params(
        ("x-subdomain" = String, 
        Header,
        description = "'x-subdomain' header represents the name of the subdomain on which the action is to be performed."),
    ),
    responses(
        (status = 204, description = "Origins were successfully deleted for subdomain."),
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
) -> Result<impl IntoResponse, DeleteOriginsError> {
    tracing::trace!(
        %subdomain.name,
        %subdomain.id,
        %user.id,
        "Deleting all origins for subdomain...",
    );

    let rows_affected = CorsService::delete_origins_for(subdomain.id, state.connection()).await?;
    tracing::trace!(
        %rows_affected,
        %subdomain.name,
        %subdomain.id,
        %user.id,
        "Origins were successfully deleted!",
    );

    Ok(StatusCode::NO_CONTENT)
}
