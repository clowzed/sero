use axum::http::StatusCode;
use sea_orm::DbErr;

#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    #[error(transparent)]
    DatabaseError(#[from] DbErr),
    #[error("Subdomain with name {0} was not found!")]
    SubdomainWasNotFound(String),
    #[error("Origin with id = {0} was not found!")]
    OriginWasNotFound(i64),
    #[error("Origin with id = {0} does not belong to subdomain with id {1}!")]
    OriginDoesNotBelongToSubdomain(i64, i64),
}

impl From<ServiceError> for StatusCode {
    fn from(value: ServiceError) -> Self {
        match value {
            ServiceError::DatabaseError(_) => Self::INTERNAL_SERVER_ERROR,
            ServiceError::SubdomainWasNotFound(_) => Self::NOT_FOUND,
            ServiceError::OriginWasNotFound(_) => Self::NOT_FOUND,
            ServiceError::OriginDoesNotBelongToSubdomain(_, _) => Self::FORBIDDEN,
        }
    }
}
