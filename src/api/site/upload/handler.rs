use super::error::UploadError;
use crate::{
    archive::parameters::UploadParameters,
    extractors::*,
    services::{archive::service::Service as ArchiveService, site::service::Service as SiteService},
    site::parameters::AssociateParameters,
    state::State as AppState,
};
use axum::{extract::State, response::IntoResponse};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use bytes::Bytes;
use sea_orm::TransactionTrait;
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(TryFromMultipart, ToSchema)]
pub struct UploadData {
    //? This will be handled in DefaultBodyLimiter
    //? And also by nginx
    #[form_data(limit = "unlimited")]
    #[schema(value_type = String, format = Binary)]
    pub archive: FieldData<Bytes>,
}

/// Uploads site for a specified subdomain.
/// Warning: Old files will be removed after successful upload.
/// The cleanup task is configured with `CLEAN_OBSOLETE_INTERVAL` env
/// If upload fails then old files will be preserved.
/// If upload fails on th stage of extracting zips then
/// new subdomain will be associated with user
///
/// Upload guard checks amount of uploads available for user.
/// The guard is configured with `MAX_SITES_PER_USER` env.
#[utoipa::path(
    tag = "Actions",
    operation_id = "Upload site",
    post,
    path = "/api/site",
    params(
        ("x-subdomain" = String, 
        Header,
        description = "x-subdomain header represents name of subdomain to call action on"),
      ),
      request_body(content = UploadData, content_type = "multipart/form-data"),
      responses(
        (status = 204, description = "Site was successfully uploaded"),
        (status = 401, description = "Unauthorized: The JWT in the header is invalid or expired.",                          body = Details),
        (status = 403, description = "Forbidden: The subdomain is owned by another user.",                                  body = Details),
        (status = 400, description = "The 'x-subdomain' header is missing or contains invalid characters.",                 body = Details),
        (status = 404, description = "Not Found: The login or subdomain was not found. See details for more information.",  body = Details),
        (status = 500, description = "Internal Server Error: An error occurred on the server.",                             body = Details),
    ),
    security(("Bearer-JWT" = []))
)]
#[tracing::instrument(skip(state, archive, user))]
pub async fn implementation(
    _: UploadGuard,
    State(state): State<Arc<AppState>>,
    AuthJWT(user): AuthJWT,
    SubdomainName(subdomain_name): SubdomainName,
    TypedMultipart(UploadData { archive }): TypedMultipart<UploadData>,
) -> Result<impl IntoResponse, UploadError> {
    let transaction = state.connection().begin().await?;

    tracing::trace!(
        %subdomain_name, 
        %user.id,
        "Associating user with subdomain...");

    let parameters = AssociateParameters {
        user_id: user.id,
        subdomain_name,
    };

    let subdomain = SiteService::grant_possession(parameters, &transaction).await?;
    tracing::trace!(%subdomain.id, 
                    %subdomain.name,        
                    %user.id,
                    "User was successfully associated with subdomain!");

    let upload_parameters = UploadParameters {
        subdomain_id: subdomain.id,
        contents: archive.contents,
        upload_folder: state.configuration().upload_folder(),
    };

    tracing::trace!(%subdomain.id, 
        %subdomain.name,        
        %user.id,
        "Starting upload process!");

    ArchiveService::upload(upload_parameters, &transaction).await?;

    tracing::trace!(%subdomain.id, 
        %subdomain.name,        
        %user.id,
        "Site was successfully uploaded!");

    transaction.commit().await?;

    Ok(())
}
