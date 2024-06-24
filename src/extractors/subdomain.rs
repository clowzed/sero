use crate::{state::State, Details};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use entity::prelude::*;
use sea_orm::prelude::*;
use std::sync::Arc;

pub struct Subdomain(pub SubdomainModel);

#[derive(thiserror::Error, Debug)]
pub enum SubdomainError {
    #[error("x-subdomain header missing")]
    XSubdomainHeaderMissing,
    #[error("x-subdomain header contains bad characters")]
    XSubdomainHeaderContainsBadChars,
    #[error("Subdomain provided in x-subdomain header was not found")]
    SubdomainWasNotFound,
    #[error(transparent)]
    DatabaseError(#[from] DbErr),
}

impl From<SubdomainError> for StatusCode {
    fn from(value: SubdomainError) -> Self {
        match value {
            SubdomainError::XSubdomainHeaderMissing | SubdomainError::XSubdomainHeaderContainsBadChars => {
                StatusCode::BAD_REQUEST
            }
            SubdomainError::SubdomainWasNotFound => StatusCode::NOT_FOUND,
            SubdomainError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for SubdomainError {
    fn into_response(self) -> Response {
        let reason = self.to_string();
        let status_code: StatusCode = self.into();

        (status_code, Json(Details { reason })).into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Subdomain
where
    Arc<State>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = SubdomainError;

    #[tracing::instrument(skip(parts, state))]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        //? Extracting app state to use database connection
        let app_state = Arc::from_ref(state);

        //? Extracting x-subdomain header and converting to string
        let header = parts
            .headers
            .get("x-subdomain")
            .ok_or(SubdomainError::XSubdomainHeaderMissing)?
            .to_str()
            .map_err(|_| SubdomainError::XSubdomainHeaderContainsBadChars)?
            .to_ascii_lowercase();

        //? Check if header is empty
        let header = match header.is_empty() {
            true => Err(SubdomainError::XSubdomainHeaderMissing),
            false => Ok(header),
        }?;

        //? Lookup for
        match SubdomainEntity::find()
            .filter(SubdomainColumn::Name.eq(&header))
            .one(app_state.connection())
            .await?
        {
            Some(model) => Ok(Self(model)),
            None => Err(SubdomainError::SubdomainWasNotFound),
        }
    }
}
