use crate::{
    auth::parameters::JwtDecodingParameters,
    services::auth::{error::ServiceError as AuthServiceError, service::Service as AuthService},
    state::State,
    Details,
};
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

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Authorization header missing.")]
    AuthorizationHeaderMissing,

    #[error("Authorization header contains bad characters.")]
    AuthorizationHeaderContainsBadChars,

    #[error("Authorization header does not schema.")]
    AuthorizationHeaderBadSchema,

    #[error(transparent)]
    AuthServiceError(#[from] AuthServiceError),

    #[error("User was not found")]
    UserWasNotFound,

    #[error(transparent)]
    DatabaseError(#[from] DbErr),
}

impl From<AuthError> for StatusCode {
    fn from(value: AuthError) -> Self {
        match value {
            AuthError::AuthorizationHeaderMissing => StatusCode::UNAUTHORIZED,
            AuthError::AuthorizationHeaderContainsBadChars | AuthError::AuthorizationHeaderBadSchema => {
                StatusCode::BAD_REQUEST
            }
            AuthError::AuthServiceError(error) => match error {
                AuthServiceError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                AuthServiceError::JwtError(_error) => StatusCode::UNAUTHORIZED,
                AuthServiceError::PasswordHashError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                AuthServiceError::LoginOccupied => StatusCode::CONFLICT,
                AuthServiceError::UserWasNotFound => StatusCode::NOT_FOUND,
            },
            AuthError::UserWasNotFound => StatusCode::UNAUTHORIZED,
            AuthError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let reason = self.to_string();
        let status_code: StatusCode = self.into();

        (status_code, Json(Details { reason })).into_response()
    }
}

pub struct AuthJWT(pub UserModel);

#[async_trait]
impl<S> FromRequestParts<S> for AuthJWT
where
    Arc<State>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    #[tracing::instrument(skip(parts, state))]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = Arc::from_ref(state);

        let auth_header_value = parts
            .headers
            .get("Authorization")
            .ok_or(AuthError::AuthorizationHeaderMissing)?
            .to_str()
            .map_err(|_| AuthError::AuthorizationHeaderContainsBadChars)?;

        let token = match auth_header_value.split_once(' ') {
            Some(("Bearer", contents)) => Ok(contents.to_owned()),
            _ => Err(AuthError::AuthorizationHeaderBadSchema),
        }?;

        let parameters = JwtDecodingParameters {
            token: token.as_ref(),
            secret: app_state.configuration().jwt_secret(),
        };

        let claims = AuthService::decode_jwt(parameters)?;

        Ok(Self({
            match UserEntity::find_by_id(claims.sub).one(app_state.connection()).await? {
                Some(user) => Ok(user),
                None => Err(AuthError::UserWasNotFound),
            }?
        }))
    }
}
