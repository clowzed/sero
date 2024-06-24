use crate::{services::site::error::ServiceError as SiteServiceError, Details};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::io;

#[derive(thiserror::Error, Debug)]
pub enum PageError {
    #[error(transparent)]
    SiteServiceError(#[from] SiteServiceError),
    #[error(transparent)]
    IoError(#[from] io::Error),
}

impl From<PageError> for StatusCode {
    fn from(value: PageError) -> Self {
        match value {
            PageError::SiteServiceError(error) => Self::from(error),
            PageError::IoError(_) => Self::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for PageError {
    fn into_response(self) -> Response {
        let reason = self.to_string();
        let status_code: StatusCode = self.into();

        (status_code, Json(Details { reason })).into_response()
    }
}
