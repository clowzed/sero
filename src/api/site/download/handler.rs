use super::error::DownloadError;
use crate::{
    extractors::*, services::site::service::Service as SiteService, site::parameters::ActionParameters,
    state::State as AppState,
};
use axum::{body::Body, extract::State, response::IntoResponse};
use std::sync::Arc;
use tokio::fs;
use tokio_util::io::ReaderStream;

/// Download site of the specified subdomain.
/// Returns a zip file which was uploaded by user (last)
#[utoipa::path(
    tag = "Actions",
    operation_id = "Download site",
    get,
    path = "/api/site",
    params(
        ("x-subdomain" = String, 
        Header,
        description = "x-subdomain header represents name of subdomain to call action on"),
      ),
    responses(
        (status = 200, description = "Site was successfully downloaded", body = String, content_type = "application/octet-stream"),
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
) -> Result<impl IntoResponse, DownloadError> {
    let parameters = ActionParameters {
        subdomain_id: subdomain.id,
    };

    tracing::info!(%subdomain.name, 
                   %subdomain.id,
                   %user.id,
                   "Retrieving site archive filepath...");

    let path = SiteService::archive(parameters, state.connection()).await?;
    tracing::info!(%subdomain.name, 
                   %subdomain.id, 
                   %user.id,
                   %path, "Site archive filepath was successfully retrieved!");

    Ok(Body::from_stream(ReaderStream::new(
        fs::File::open(&path).await.inspect_err(|cause| {
            tracing::info!(%cause, 
                           %subdomain.name, 
                           %subdomain.id, 
                           %user.id,
                           %path, 
                           "Site archive filepath was successfully retrieved!")
        })?,
    )))
}
