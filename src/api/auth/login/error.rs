use crate::{services::auth::error::ServiceError as AuthServiceError, Details};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use validator::ValidationErrors;

#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    #[error(transparent)]
    AuthServiceError(#[from] AuthServiceError),
    #[error("Login or password does not match validation rules!")]
    ValidationError(#[from] ValidationErrors),
}

impl From<LoginError> for StatusCode {
    fn from(value: LoginError) -> Self {
        match value {
            LoginError::AuthServiceError(error) => Self::from(error),
            LoginError::ValidationError(_) => Self::BAD_REQUEST,
        }
    }
}

impl IntoResponse for LoginError {
    fn into_response(self) -> Response {
        let reason = self.to_string();
        let status_code: StatusCode = self.into();

        (status_code, Json(Details { reason })).into_response()
    }
}
