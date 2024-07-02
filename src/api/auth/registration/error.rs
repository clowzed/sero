use crate::{services::auth::error::ServiceError as AuthServiceError, Details};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::DbErr;
use validator::ValidationErrors;

#[derive(thiserror::Error, Debug)]
pub enum RegistrationError {
    #[error(transparent)]
    AuthServiceError(#[from] AuthServiceError),
    #[error(transparent)]
    DatabaseError(#[from] DbErr),
    #[error("Login or password does not match validation rules!")]
    ValidationError(#[from] ValidationErrors),
}

impl From<RegistrationError> for StatusCode {
    fn from(value: RegistrationError) -> Self {
        match value {
            RegistrationError::AuthServiceError(error) => Self::from(error),
            RegistrationError::DatabaseError(_error) => StatusCode::INTERNAL_SERVER_ERROR,
            RegistrationError::ValidationError(_) => Self::BAD_REQUEST,
        }
    }
}

impl IntoResponse for RegistrationError {
    fn into_response(self) -> Response {
        let reason = self.to_string();
        let status_code: StatusCode = self.into();

        tracing::error!(%reason, %status_code, "Error occurred while trying to handle request!");
        (status_code, Json(Details { reason })).into_response()
    }
}
