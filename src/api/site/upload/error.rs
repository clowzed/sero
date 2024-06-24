use crate::{
    services::{archive::error::ServiceError as ArchiveServiceError, site::error::ServiceError as SiteServiceError},
    Details,
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::DbErr;

#[derive(thiserror::Error, Debug)]
pub enum UploadError {
    #[error(transparent)]
    DbError(#[from] DbErr),
    #[error(transparent)]
    SiteServiceError(#[from] SiteServiceError),
    #[error(transparent)]
    ArchiveServiceError(#[from] ArchiveServiceError),
}

impl From<UploadError> for StatusCode {
    fn from(value: UploadError) -> Self {
        match value {
            UploadError::DbError(_) => Self::INTERNAL_SERVER_ERROR,
            UploadError::SiteServiceError(error) => Self::from(error),
            UploadError::ArchiveServiceError(error) => Self::from(error),
        }
    }
}

impl IntoResponse for UploadError {
    fn into_response(self) -> Response {
        let reason = self.to_string();
        let status_code: StatusCode = self.into();

        (status_code, Json(Details { reason })).into_response()
    }
}
