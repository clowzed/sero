use super::error::DeleteOriginError;
use crate::{extractors::*, services::origin::service::Service as CorsService, state::State as AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use sea_orm::TransactionTrait;
use std::sync::Arc;

/// Delete origin by id for specified subdomain for dynamic CORS (Cross-Origin Resource Sharing) management.
///
/// This endpoint allows users to delete origin by id that is permitted to access resources
/// on their specified subdomains. The action is authenticated using a JWT, and the subdomain must
/// be owned by the user making the request. This will be checked by the server.
#[utoipa::path(
    delete,
    tag = "Origins Management and Dynamic Access Control",
    operation_id = "Delete origin by id",
    path = "/api/origin/{id}",
    params(
        ("x-subdomain" = String, 
        Header,
        description = "'x-subdomain' header represents the name of the subdomain on which the action is to be performed."),
        ("id" = i64, Path, description = "Id of the origin to delete"),
    ),
    responses(
        (status = 204, description = "Origin was successfully deleted for subdomain."),
        (status = 400, description = "The 'x-subdomain' header is missing or contains invalid characters.",                           body = Details),
        (status = 401, description = "Unauthorized: The JWT in the header is invalid or expired.",                                    body = Details),
        (status = 403, description = "Forbidden: The origin is owned by another user.",                                               body = Details),
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
) -> Result<impl IntoResponse, DeleteOriginError> {
    let transaction = state.connection().begin().await?;
    tracing::trace!(
        %origin_id,
        %subdomain.name,
        %subdomain.id,
        %user.id,
        "Deleting origin for subdomain...",
    );

    let rows_affected = CorsService::delete_origin_of(subdomain.id, origin_id, &transaction).await?;

    tracing::trace!(
        %origin_id,
        %subdomain.name,
        %subdomain.id,
        %rows_affected,
        %user.id,
        "Origin was successfully deleted for subdomain. Committing changes...",
    );

    transaction.commit().await?;
    tracing::trace!( %origin_id,
        %subdomain.name,
        %subdomain.id,
        %user.id,
        %rows_affected,
        "Origin was successfully deleted for subdomain. Changes were successfully committed!");

    Ok(StatusCode::NO_CONTENT)
}
