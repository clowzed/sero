use crate::{services::site::error::ServiceError as SiteServiceError, Details};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::DbErr;

#[derive(thiserror::Error, Debug)]
pub enum TeardownError {
    #[error(transparent)]
    DbError(#[from] DbErr),
    #[error(transparent)]
    SiteServiceError(#[from] SiteServiceError),
}

impl From<TeardownError> for StatusCode {
    fn from(value: TeardownError) -> Self {
        match value {
            TeardownError::DbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TeardownError::SiteServiceError(error) => Self::from(error),
        }
    }
}

impl IntoResponse for TeardownError {
    fn into_response(self) -> Response {
        let reason = self.to_string();
        let status_code: StatusCode = self.into();

        (status_code, Json(Details { reason })).into_response()
    }
}
