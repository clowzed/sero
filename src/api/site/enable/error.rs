use crate::{services::site::error::ServiceError as SiteServiceError, Details};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::DbErr;

#[derive(thiserror::Error, Debug)]
pub enum EnableError {
    #[error(transparent)]
    DbError(#[from] DbErr),
    #[error(transparent)]
    SiteServiceError(#[from] SiteServiceError),
}

impl From<EnableError> for StatusCode {
    fn from(value: EnableError) -> Self {
        match value {
            EnableError::DbError(_) => Self::INTERNAL_SERVER_ERROR,
            EnableError::SiteServiceError(error) => Self::from(error),
        }
    }
}

impl IntoResponse for EnableError {
    fn into_response(self) -> Response {
        let reason = self.to_string();
        let status_code: StatusCode = self.into();

        (status_code, Json(Details { reason })).into_response()
    }
}
