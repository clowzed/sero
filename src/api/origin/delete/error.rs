use crate::{services::origin::error::ServiceError as OriginServiceError, Details};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::DbErr;

#[derive(thiserror::Error, Debug)]
pub enum DeleteOriginError {
    #[error(transparent)]
    OriginServiceError(#[from] OriginServiceError),
    #[error(transparent)]
    DatabaseError(#[from] DbErr),
}

impl From<DeleteOriginError> for StatusCode {
    fn from(value: DeleteOriginError) -> Self {
        match value {
            DeleteOriginError::OriginServiceError(error) => Self::from(error),
            DeleteOriginError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for DeleteOriginError {
    fn into_response(self) -> Response {
        let reason = self.to_string();
        let status_code: StatusCode = self.into();

        tracing::error!(%reason, %status_code, "Error occurred while trying to handle request!");
        (status_code, Json(Details { reason })).into_response()
    }
}
