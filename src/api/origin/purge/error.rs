use crate::{services::origin::error::ServiceError as OriginServiceError, Details};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(thiserror::Error, Debug)]
pub enum DeleteOriginsError {
    #[error(transparent)]
    OriginServiceError(#[from] OriginServiceError),
}

impl From<DeleteOriginsError> for StatusCode {
    fn from(value: DeleteOriginsError) -> Self {
        match value {
            DeleteOriginsError::OriginServiceError(error) => Self::from(error),
        }
    }
}

impl IntoResponse for DeleteOriginsError {
    fn into_response(self) -> Response {
        let reason = self.to_string();
        let status_code: StatusCode = self.into();

        (status_code, Json(Details { reason })).into_response()
    }
}
