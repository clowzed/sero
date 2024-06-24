use super::super::{
    auth::{AuthError, AuthJWT},
    subdomain_name::{SubdomainName, SubdomainNameError},
};
use crate::{state::State, Details};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use entity::prelude::*;
use sea_orm::{prelude::*, query::*, FromQueryResult};
use std::sync::Arc;

pub struct Guard;

#[derive(FromQueryResult)]
struct SubdomainCountQueryResult {
    count: Option<i64>,
}

#[derive(thiserror::Error, Debug)]
pub enum GuardError {
    #[error("Max sites per user limit exceeded")]
    LimitExceeded,
    #[error(transparent)]
    DatabaseError(#[from] DbErr),
    #[error(transparent)]
    AuthError(#[from] AuthError),
    #[error(transparent)]
    SubdomainNameError(#[from] SubdomainNameError),
}

impl From<GuardError> for StatusCode {
    fn from(value: GuardError) -> Self {
        match value {
            GuardError::LimitExceeded => StatusCode::FORBIDDEN,
            GuardError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            GuardError::AuthError(error) => StatusCode::from(error),
            GuardError::SubdomainNameError(error) => StatusCode::from(error),
        }
    }
}

impl IntoResponse for GuardError {
    fn into_response(self) -> Response {
        let reason = self.to_string();
        let status_code: StatusCode = self.into();

        (status_code, Json(Details { reason })).into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Guard
where
    Arc<State>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = GuardError;

    #[tracing::instrument(skip(parts, state))]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = Arc::from_ref(state);

        let user = AuthJWT::from_request_parts(parts, state).await?.0;
        let subdomain = SubdomainName::from_request_parts(parts, state).await?.0;

        match app_state.configuration().max_sites_per_user() {
            Some(max_sites) => {
                let users_sites_count = SubdomainEntity::find()
                    .filter(SubdomainColumn::OwnerId.eq(user.id))
                    .select_only()
                    .column_as(
                        Expr::expr(Expr::case(Expr::col(SubdomainColumn::Name).eq(&subdomain), 0).finally(1)).sum(),
                        "count",
                    )
                    .into_model::<SubdomainCountQueryResult>()
                    .one(app_state.connection())
                    .await?
                    .unwrap_or(SubdomainCountQueryResult { count: Some(i64::MAX) })
                    .count;

                match users_sites_count.unwrap_or(0) as u64 >= max_sites {
                    true => Err(GuardError::LimitExceeded),
                    false => Ok(Self {}),
                }
            }
            None => Ok(Self {}),
        }
    }
}
