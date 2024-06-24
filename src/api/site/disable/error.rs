use crate::{services::site::error::ServiceError as SiteServiceError, Details};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::DbErr;
#[derive(thiserror::Error, Debug)]
pub enum DisableError {
    #[error(transparent)]
    DbError(#[from] DbErr),
    #[error(transparent)]
    SiteServiceError(#[from] SiteServiceError),
}

impl From<DisableError> for StatusCode {
    fn from(value: DisableError) -> Self {
        match value {
            DisableError::DbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DisableError::SiteServiceError(error) => Self::from(error),
        }
    }
}

impl IntoResponse for DisableError {
    fn into_response(self) -> Response {
        let reason = self.to_string();
        let status_code: StatusCode = self.into();

        (status_code, Json(Details { reason })).into_response()
    }
}
