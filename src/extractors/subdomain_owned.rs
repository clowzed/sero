use super::{auth::AuthError, subdomain::SubdomainError, AuthJWT, Subdomain};
use crate::{state::State, Details};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use entity::prelude::*;
use std::sync::Arc;

#[derive(Debug)]
pub struct SubdomainOwned {
    pub user: UserModel,
    pub subdomain: SubdomainModel,
}

#[derive(thiserror::Error, Debug)]
pub enum SubdomainOwnedError {
    #[error(transparent)]
    SubdomainError(#[from] SubdomainError),
    #[error(transparent)]
    AuthError(#[from] AuthError),
    #[error("Subdomain provided in X-Subdomain header is owned by another user")]
    SubdomainIsOwnedByAnotherUser,
}

impl From<SubdomainOwnedError> for StatusCode {
    fn from(value: SubdomainOwnedError) -> Self {
        match value {
            SubdomainOwnedError::SubdomainError(error) => StatusCode::from(error),
            SubdomainOwnedError::AuthError(error) => StatusCode::from(error),
            SubdomainOwnedError::SubdomainIsOwnedByAnotherUser => StatusCode::FORBIDDEN,
        }
    }
}

impl IntoResponse for SubdomainOwnedError {
    fn into_response(self) -> Response {
        let reason = self.to_string();
        let status_code: StatusCode = self.into();

        tracing::error!(%reason, %status_code, "Error occurred while trying to handle request!");
        (status_code, Json(Details { reason })).into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for SubdomainOwned
where
    Arc<State>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = SubdomainOwnedError;

    #[tracing::instrument(skip(parts, state))]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let subdomain = Subdomain::from_request_parts(parts, state).await?.0;
        let user = AuthJWT::from_request_parts(parts, state).await?.0;

        match subdomain.owner_id == user.id {
            true => Ok(Self { user, subdomain }),
            false => Err(SubdomainOwnedError::SubdomainIsOwnedByAnotherUser),
        }
    }
}
