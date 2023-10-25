use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use sea_orm::prelude::*;
use std::sync::Arc;

use crate::{apperror::SeroError, services::users::UsersService};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct TokenClaims {
    pub sub: i32,
    pub iat: u64,
    pub exp: u64,
}

pub struct Subdomain(pub String);

pub struct SubdomainModel(pub entity::prelude::Subdomain);
pub struct AuthJWT(pub entity::user::Model);

pub struct RegistrationGuard;

#[async_trait]
impl<S> FromRequestParts<S> for AuthJWT
where
    Arc<crate::AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = SeroError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = Arc::from_ref(state);

        let auth_header_value = parts
            .headers
            .get("Authorization")
            .ok_or(SeroError::AuthorizationHeaderMissing)?
            .to_str()
            .map_err(|_| SeroError::AuthorizationHeaderBabChars)?;

        let token = match auth_header_value.split_once(' ') {
            Some(("Bearer", contents)) => Ok(contents.to_string()),
            _ => Err(SeroError::AuthorizationHeaderBadSchema),
        }?;

        match crate::services::auth::AuthService::jwtcheck(
            &token,
            &app_state.connection,
            app_state.config.jwt_secret.as_ref().unwrap(),
        )
        .await
        {
            Ok(Some(user)) => Ok(Self(user)),
            Ok(None) => Err(SeroError::UserWasNotFoundUsingJwt),
            Err(cause) => Err(SeroError::InternalServerError(Box::new(cause))),
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Subdomain
where
    Arc<crate::AppState>: FromRef<S>,

    S: Send + Sync,
{
    type Rejection = SeroError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self({
            let header = parts
                .headers
                .get("X-Subdomain")
                .ok_or(SeroError::XSubdomainHeaderMissing)?
                .to_str()
                .map_err(|_| SeroError::XSubdomainHeaderMissing)?
                .to_string();

            match header.is_empty() {
                true => Err(SeroError::XSubdomainHeaderMissing)?,
                false => header,
            }
        }))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for SubdomainModel
where
    Arc<crate::AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = SeroError;

    #[tracing::instrument(skip(parts, state))]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = Arc::from_ref(state);

        let subdomain_name = Subdomain::from_request_parts(parts, state).await?.0;
        Ok(match entity::prelude::SubdomainEntity::find()
            .filter(entity::prelude::SubdomainColumn::Name.eq(&subdomain_name))
            .one(&app_state.connection)
            .await
        {
            Ok(Some(subdomain)) => Ok(Self(subdomain)),
            Ok(None) => Err(SeroError::SubdomainWasNotFound(subdomain_name)),
            Err(cause) => Err(SeroError::InternalServerError(Box::new(cause))),
        }?)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for RegistrationGuard
where
    Arc<crate::AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = SeroError;

    #[tracing::instrument(skip(_parts, state))]
    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = Arc::from_ref(state);

        match UsersService::count(&app_state.connection).await {
            Ok(count) => match app_state.config.max_users {
                Some(max_users) if count > max_users => Err(SeroError::RegisteredUserLimitExceeded),
                _ => Ok(Self {}),
            },
            Err(cause) => Err(SeroError::InternalServerError(Box::new(cause))),
        }
    }
}
