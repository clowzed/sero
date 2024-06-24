use crate::{services::site::error::ServiceError as SiteServiceError, Details};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::DbErr;
use std::io;

#[derive(thiserror::Error, Debug)]
pub enum DownloadError {
    #[error(transparent)]
    DbError(#[from] DbErr),
    #[error(transparent)]
    SiteServiceError(#[from] SiteServiceError),
    #[error(transparent)]
    IoError(#[from] io::Error),
}

impl From<DownloadError> for StatusCode {
    fn from(value: DownloadError) -> Self {
        match value {
            DownloadError::DbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DownloadError::SiteServiceError(error) => Self::from(error),
            DownloadError::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for DownloadError {
    fn into_response(self) -> Response {
        let reason = self.to_string();
        let status_code: StatusCode = self.into();

        (status_code, Json(Details { reason })).into_response()
    }
}
