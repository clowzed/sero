use axum::http::StatusCode;

#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    #[error("Zip archive is empty!")]
    EmptyArchive,
    #[error("Subdomain with id = {0} was not found!")]
    SubdomainWasNotFound(i64),
    #[error(transparent)]
    ZipError(#[from] async_zip::error::ZipError),
    #[error(transparent)]
    FileSystemError(#[from] tokio::io::Error),
    #[error(transparent)]
    DatabaseError(#[from] sea_orm::DbErr),
}

impl From<ServiceError> for StatusCode {
    fn from(value: ServiceError) -> Self {
        match value {
            ServiceError::EmptyArchive => Self::BAD_REQUEST,
            ServiceError::SubdomainWasNotFound(_) => Self::NOT_FOUND,
            ServiceError::ZipError(_) => Self::BAD_REQUEST,
            ServiceError::FileSystemError(_) => Self::INTERNAL_SERVER_ERROR,
            ServiceError::DatabaseError(_) => Self::INTERNAL_SERVER_ERROR,
        }
    }
}
