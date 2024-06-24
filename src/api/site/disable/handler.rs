use super::error::DisableError;
use crate::{
    extractors::*, services::site::service::Service as SiteService, site::parameters::ActionParameters,
    state::State as AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use sea_orm::TransactionTrait;
use std::sync::Arc;

/// Disables a specific site identified by the `x-subdomain` header.
///
/// This endpoint allows authenticated users to disable a site associated with the specified subdomain.
#[utoipa::path(
    tag = "Actions",
    operation_id = "Disable site",
    patch,
    path = "/api/site/disable",
    params(
        ("x-subdomain" = String, 
        Header,
        description = "x-subdomain header represents name of subdomain to call action on"),
      ),
    responses(
        (status = 204, description = "Site was successfully disabled."),
        (status = 401, description = "Unauthorized: The JWT in the header is invalid or expired.",                          body = Details),
        (status = 403, description = "Forbidden: The subdomain is owned by another user.",                                  body = Details),
        (status = 400, description = "The 'x-subdomain' header is missing or contains invalid characters.",                 body = Details),
        (status = 404, description = "Not Found: The login or subdomain was not found. See details for more information.",  body = Details),
        (status = 500, description = "Internal Server Error: An error occurred on the server.",                             body = Details),
    ),
    security(("Bearer-JWT" = []))
)]
#[tracing::instrument(skip(state))]
pub async fn implementation(
    State(state): State<Arc<AppState>>,
    SubdomainOwned { user, subdomain }: SubdomainOwned,
) -> Result<impl IntoResponse, DisableError> {
    tracing::trace!(%subdomain.name, 
                    %subdomain.id,
                    %user.id,
                    "Disabling site...");

    let transaction = state.connection().begin().await?;

    let parameters = ActionParameters {
        subdomain_id: subdomain.id,
    };
    SiteService::disable(parameters, &transaction).await?;
    tracing::trace!(%subdomain.name, 
                    %subdomain.id, 
                    %user.id,
                    "Site was successfully disabled. Committing changes...");

    transaction.commit().await?;
    tracing::trace!(%subdomain.name, 
                    %subdomain.id, 
                    %user.id,
                    "Site was successfully disabled. Successfully committed!");

    Ok(StatusCode::NO_CONTENT)
}
