use axum::http::StatusCode;
use sea_orm::DbErr;
use std::fmt::Debug;

#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    #[error(transparent)]
    FileSystemError(#[from] tokio::io::Error),
    #[error(transparent)]
    DatabaseError(#[from] DbErr),
    #[error("No archive is related to this subdomain")]
    ArchiveNotFound,
    #[error("Subdomain provided in x-subdomain header is owned by another user")]
    SubdomainIsOwnedByAnotherUser,
    #[error("Subdomain provided in x-subdomain header was not found")]
    SubdomainWasNotFound,
}

impl From<ServiceError> for StatusCode {
    fn from(value: ServiceError) -> Self {
        match value {
            ServiceError::FileSystemError(_) => Self::INTERNAL_SERVER_ERROR,
            ServiceError::DatabaseError(_) => Self::INTERNAL_SERVER_ERROR,
            ServiceError::ArchiveNotFound => Self::NOT_FOUND,
            ServiceError::SubdomainIsOwnedByAnotherUser => Self::FORBIDDEN,
            ServiceError::SubdomainWasNotFound => Self::NOT_FOUND,
        }
    }
}
