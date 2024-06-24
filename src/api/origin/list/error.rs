use crate::{services::origin::error::ServiceError as OriginServiceError, Details};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(thiserror::Error, Debug)]
pub enum ListOriginsError {
    #[error(transparent)]
    OriginServiceError(#[from] OriginServiceError),
}

impl From<ListOriginsError> for StatusCode {
    fn from(value: ListOriginsError) -> Self {
        match value {
            ListOriginsError::OriginServiceError(error) => Self::from(error),
        }
    }
}

impl IntoResponse for ListOriginsError {
    fn into_response(self) -> Response {
        let reason = self.to_string();
        let status_code: StatusCode = self.into();

        (status_code, Json(Details { reason })).into_response()
    }
}
