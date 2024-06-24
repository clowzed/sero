use super::{error::ServiceError, parameters::*};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use chrono::{prelude::*, Duration};
use entity::prelude::*;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use sea_orm::{prelude::*, Set, TransactionTrait};

pub struct Service;

impl Service {
    #[tracing::instrument(skip(parameters))]
    pub fn generate_jwt<T>(user_id: i64, parameters: JwtGenerationParameters<T>) -> Result<String, ServiceError>
    where
        T: AsRef<str>,
    {
        let claims: TokenClaims = TokenClaims {
            sub: user_id,
            exp: (Utc::now() + Duration::try_seconds(parameters.ttl).unwrap_or_default()).timestamp(),
            iat: Utc::now().timestamp(),
        };

        Ok(jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(parameters.secret.as_ref().as_bytes()),
        )?)
    }

    #[tracing::instrument(skip(parameters))]
    pub fn decode_jwt<T>(parameters: JwtDecodingParameters<T>) -> Result<TokenClaims, ServiceError>
    where
        T: AsRef<str>,
    {
        let validation = Validation::default();

        let decoded = jsonwebtoken::decode::<TokenClaims>(
            parameters.token.as_ref(),
            &DecodingKey::from_secret(parameters.secret.as_ref().as_bytes()),
            &validation,
        )?;

        Ok(decoded.claims)
    }

    #[tracing::instrument(skip(connection, credentials))]
    pub async fn login<T, C>(credentials: UserCredentials<T>, connection: &C) -> Result<UserModel, ServiceError>
    where
        C: ConnectionTrait + TransactionTrait,
        T: AsRef<str>,
    {
        let user = match UserEntity::find()
            .filter(UserColumn::Login.eq(credentials.login.as_ref()))
            .one(connection)
            .await?
        {
            Some(user) => Ok(user),
            None => Err(ServiceError::UserWasNotFound),
        }?;

        let parsed_hash = PasswordHash::new(&user.password)?;

        Argon2::default().verify_password(credentials.password.as_ref().as_bytes(), &parsed_hash)?;

        Ok(user)
    }

    #[tracing::instrument(skip(connection, credentials))]
    pub async fn registration<T, C>(credentials: UserCredentials<T>, connection: &C) -> Result<UserModel, ServiceError>
    where
        C: ConnectionTrait + TransactionTrait,
        T: AsRef<str>,
    {
        match UserEntity::find()
            .filter(UserColumn::Login.eq(credentials.login.as_ref()))
            .one(connection)
            .await?
        {
            Some(_) => Err(ServiceError::LoginOccupied),
            None => {
                let salt = SaltString::generate(&mut OsRng);

                let hashed_password = Argon2::default()
                    .hash_password(credentials.password.as_ref().as_bytes(), &salt)?
                    .to_string();

                let user_to_be_inserted = UserActiveModel {
                    login: Set(credentials.login.as_ref().to_owned()),
                    password: Set(hashed_password),
                    ..Default::default()
                };

                Ok(UserEntity::insert(user_to_be_inserted)
                    .exec_with_returning(connection)
                    .await?)
            }
        }
    }
}
