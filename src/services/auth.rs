use crate::extractors::TokenClaims;
use crate::services::users::UsersService;
use chrono::Duration;
use chrono::Utc;
use entity::prelude::*;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use sea_orm::DbErr;
use thiserror::Error;

use super::users::UserCredentials;
pub type Jwt = String;

#[derive(Error, Debug)]
pub enum AuthError {}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AuthCredentials {
    pub username: String,
    pub password: String,
}

impl AuthCredentials {
    pub fn random() -> Self {
        Self {
            username: uuid::Uuid::new_v4().to_string(),
            password: uuid::Uuid::new_v4().to_string(),
        }
    }

    pub fn random_unvalid() -> Self {
        Self {
            username: uuid::Uuid::new_v4().to_string(),
            password: "".to_owned(),
        }
    }
}

impl AuthCredentials {
    pub fn valid(&self) -> bool {
        !self.username.is_empty() && !self.password.is_empty()
    }
}

impl std::fmt::Debug for AuthCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthCredentials")
            .field("username", &self.username)
            .finish()
    }
}

impl From<AuthCredentials> for UserCredentials {
    fn from(val: AuthCredentials) -> Self {
        UserCredentials {
            username: val.username,
            password: val.password,
        }
    }
}
pub struct AuthService;

impl AuthService {
    #[tracing::instrument(skip(connection, key))]
    pub async fn login(
        credentials: AuthCredentials,
        connection: &sea_orm::DatabaseConnection,
        key: &str,
    ) -> Result<Option<Jwt>, migration::DbErr> {
        match UsersService::find(credentials.clone().into(), connection).await? {
            Some(user) => {
                let claims: TokenClaims = TokenClaims {
                    sub: user.id,
                    exp: (Utc::now() + Duration::minutes(10)).timestamp() as u64,
                    iat: Utc::now().timestamp() as u64,
                };

                let token = Ok(Some(
                    jsonwebtoken::encode(
                        &jsonwebtoken::Header::default(),
                        &claims,
                        &jsonwebtoken::EncodingKey::from_secret(key.as_bytes()),
                    )
                    .unwrap(),
                ));
                token
            }
            None => Ok(None),
        }
    }

    #[tracing::instrument(skip(connection, key))]
    pub async fn jwtcheck(
        token: &Jwt,
        connection: &sea_orm::DatabaseConnection,
        key: &str,
    ) -> Result<Option<User>, migration::DbErr> {
        let claims = match jsonwebtoken::decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(key.as_bytes()),
            &Validation::default(),
        ) {
            Ok(decoded) => {
                let claims = decoded.claims;
                if claims.exp <= jsonwebtoken::get_current_timestamp() {
                    return Ok(None);
                }

                claims
            }
            Err(_) => {
                return Ok(None);
            }
        };
        UsersService::find_by_id(claims.sub, connection).await
    }

    #[tracing::instrument(skip(connection))]
    pub async fn registration(
        credentials: AuthCredentials,
        connection: &sea_orm::DatabaseConnection,
    ) -> Result<Option<User>, DbErr> {
        match UsersService::find_by_username(&credentials.username, connection).await? {
            Some(_) => Ok(None),
            None => Ok(Some(
                UsersService::new_user(credentials.into(), connection).await?,
            )),
        }
    }
}
