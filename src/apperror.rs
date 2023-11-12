use std::{error::Error, fmt::Debug};

use axum::{http::StatusCode, response::IntoResponse, Json};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Details {
    details: String,
}

pub enum SeroError {
    XSubdomainHeaderMissing,
    AuthorizationHeaderMissing,
    AuthorizationHeaderBadSchema,
    AuthorizationHeaderBabChars,
    InternalServerError(Box<dyn Error>),
    SubdomainIsOwnedByAnotherUser(String),
    UserWasNotFoundUsingJwt,
    RegisteredUserLimitExceeded,
    Unauthorized,
    UserHasAlreadyBeenRegistered,
    SubdomainWasNotFound(String),
    ArchiveFileWasNotFoundForSubdomain(String),
    MaxSitesPerUserLimitExceeded,
    SiteDisabled,
    EmptyCredentials,
}

impl std::fmt::Debug for SeroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            SeroError::XSubdomainHeaderMissing => "X-Subdomain header is missing!".to_string(),
            SeroError::AuthorizationHeaderMissing => "Authorization header is missing!".to_string(),
            SeroError::AuthorizationHeaderBadSchema => "Authorization header does not match schema! Required schema: Authorization: Bearer <token>".to_string(),
            SeroError::SubdomainIsOwnedByAnotherUser(subdomain_name) => format!("Subdomain with name {} is owned by another user!", subdomain_name),
            SeroError::AuthorizationHeaderBabChars => "Authorization header contains invalid characters!".to_string(),
            SeroError::InternalServerError(cause) => {
                tracing::error!(%cause, "Error!");
                "Some error occurred on the server!".to_string()
            },
            SeroError::UserWasNotFoundUsingJwt => "User with id from jwt token was not found!".to_string(),
            SeroError::RegisteredUserLimitExceeded => "Registered user limit exceeded!".to_string(),
            SeroError::Unauthorized => "Unauthorized! Bad credentials were provided!".to_string(),
            SeroError::UserHasAlreadyBeenRegistered => "User with this username has already been registered!".to_string(),
            SeroError::SubdomainWasNotFound(subdomain_name) => format!("Subdomain with name {} was not found!", subdomain_name),
            SeroError::ArchiveFileWasNotFoundForSubdomain(subdomain_name) => format!("Archive file was not found for subdomain {}", subdomain_name),
            SeroError::MaxSitesPerUserLimitExceeded => "Max sites per this user limit exceeded!".to_string(),
            SeroError::SiteDisabled => "Site is disabled!".to_string(),
            SeroError::EmptyCredentials => "Empty credentials were provided!".to_string(),
        })
    }
}

impl std::fmt::Display for SeroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<&SeroError> for StatusCode {
    fn from(val: &SeroError) -> Self {
        match val {
            SeroError::XSubdomainHeaderMissing => StatusCode::BAD_REQUEST,
            SeroError::AuthorizationHeaderMissing => StatusCode::BAD_REQUEST,
            SeroError::AuthorizationHeaderBadSchema => StatusCode::BAD_REQUEST,
            SeroError::SubdomainIsOwnedByAnotherUser(_) => StatusCode::FORBIDDEN,
            SeroError::AuthorizationHeaderBabChars => StatusCode::BAD_REQUEST,
            SeroError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SeroError::UserWasNotFoundUsingJwt => StatusCode::UNAUTHORIZED,
            SeroError::RegisteredUserLimitExceeded => StatusCode::FORBIDDEN,
            SeroError::Unauthorized => StatusCode::UNAUTHORIZED,
            SeroError::UserHasAlreadyBeenRegistered => StatusCode::CONFLICT,
            SeroError::SubdomainWasNotFound(_) => StatusCode::NOT_FOUND,
            SeroError::ArchiveFileWasNotFoundForSubdomain(_) => StatusCode::NOT_FOUND,
            SeroError::MaxSitesPerUserLimitExceeded => StatusCode::FORBIDDEN,
            SeroError::SiteDisabled => StatusCode::SERVICE_UNAVAILABLE,
            SeroError::EmptyCredentials => StatusCode::BAD_REQUEST,
        }
    }
}

impl IntoResponse for SeroError {
    fn into_response(self) -> axum::response::Response {
        let response = (
            Into::<StatusCode>::into(&self),
            Json(Details {
                details: format!("{:?}", self),
            }),
        );
        tracing::error!(cause = response.1.details, "Response with error!");
        response.into_response()
    }
}
