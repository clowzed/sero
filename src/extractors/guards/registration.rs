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

pub struct Guard;

#[derive(thiserror::Error, Debug)]
pub enum GuardError {
    #[error("Failed to register new user because user limit exceeded!")]
    UsersLimitExceeded,
    #[error(
        "Some error occurred while communicating with database! 
        Contact with the administrator"
    )]
    DatabaseError(#[from] DbErr),
}

impl From<&GuardError> for StatusCode {
    fn from(value: &GuardError) -> Self {
        match value {
            GuardError::UsersLimitExceeded => StatusCode::FORBIDDEN,
            GuardError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for GuardError {
    fn into_response(self) -> Response {
        (
            StatusCode::from(&self),
            Json(Details {
                reason: self.to_string(),
            }),
        )
            .into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Guard
where
    Arc<State>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = GuardError;

    #[tracing::instrument(skip(_parts, state))]
    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = Arc::from_ref(state);

        match app_state.configuration().max_users() {
            Some(max_users) => match UserEntity::find().count(app_state.connection()).await {
                Ok(user_count) if user_count >= max_users => Err(GuardError::UsersLimitExceeded),
                _ => Ok(Self {}),
            },
            None => Ok(Self {}),
        }
    }
}
