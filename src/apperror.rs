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

impl IntoResponse for SeroError {
    fn into_response(self) -> axum::response::Response {
        let response = match self {
            SeroError::XSubdomainHeaderMissing => (
                StatusCode::BAD_REQUEST,
                Json(Details {
                    details: "X-Subdomain header is missing!".into(),
                }),
            ),
            SeroError::AuthorizationHeaderMissing => (
                StatusCode::BAD_REQUEST,
                Json(Details {
                    details: "Authorization header is missing!".into(),
                }),
            ),
            SeroError::AuthorizationHeaderBadSchema => (
                StatusCode::BAD_REQUEST,
                Json(Details {
                    details: "Authorization header does not match schema!
                    Required schema: Authorization: Bearer <token>"
                        .into(),
                }),
            ),
            SeroError::SubdomainIsOwnedByAnotherUser(subdomain_name) => (
                StatusCode::FORBIDDEN,
                Json(Details {
                    details: format!(
                        "Subdomain with name {} is owned by another user!",
                        subdomain_name
                    ),
                }),
            ),
            SeroError::AuthorizationHeaderBabChars => (
                StatusCode::BAD_REQUEST,
                Json(Details {
                    details: "Authorization header contains invalid characters!".into(),
                }),
            ),
            SeroError::InternalServerError(cause) => {
                tracing::error!(%cause, "Error!");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(Details {
                        details: "Some error occurred on the server!".into(),
                    }),
                )
            }
            SeroError::UserWasNotFoundUsingJwt => (
                StatusCode::UNAUTHORIZED,
                Json(Details {
                    details: "User with id from jwt token was not found!".into(),
                }),
            ),
            SeroError::RegisteredUserLimitExceeded => (
                StatusCode::FORBIDDEN,
                Json(Details {
                    details: "Registered user limit exceeded!".into(),
                }),
            ),
            SeroError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                Json(Details {
                    details: "Unauthorized! Bad credentials were provided!".into(),
                }),
            ),
            SeroError::UserHasAlreadyBeenRegistered => (
                StatusCode::CONFLICT,
                Json(Details {
                    details: "User with this username has already been registered!".into(),
                }),
            ),
            SeroError::SubdomainWasNotFound(subdomain_name) => (
                StatusCode::NOT_FOUND,
                Json(Details {
                    details: format!("Subdomain with name {subdomain_name} was not found!"),
                }),
            ),
            SeroError::ArchiveFileWasNotFoundForSubdomain(subdomain_name) => (
                StatusCode::NOT_FOUND,
                Json(Details {
                    details: format!("Archive file was not found for subdomain {subdomain_name}"),
                }),
            ),
            SeroError::MaxSitesPerUserLimitExceeded => (
                StatusCode::FORBIDDEN,
                Json(Details {
                    details: "Max sites per this user limit exceeded!".into(),
                }),
            ),
            SeroError::SiteDisabled => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(Details {
                    details: "Service is currently unavailable!".into(),
                }),
            ),
            SeroError::EmptyCredentials => (
                StatusCode::BAD_REQUEST,
                Json(Details {
                    details: "Username or password is empty!".into(),
                }),
            ),
        };

        tracing::error!(cause = response.1.details, "Response with error!");
        response.into_response()
    }
}
