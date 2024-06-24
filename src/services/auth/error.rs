use argon2::password_hash;
use axum::http::StatusCode;
use sea_orm::prelude::*;
use std::fmt::Debug;

#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    #[error(transparent)]
    DatabaseError(#[from] DbErr),
    #[error(transparent)]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error(transparent)]
    PasswordHashError(#[from] password_hash::Error),
    #[error("Login has already been occupied.")]
    LoginOccupied,
    #[error("User was not found by login.")]
    UserWasNotFound,
}

impl From<ServiceError> for StatusCode {
    fn from(value: ServiceError) -> Self {
        match value {
            ServiceError::DatabaseError(_) => Self::INTERNAL_SERVER_ERROR,
            ServiceError::JwtError(_error) => Self::UNAUTHORIZED,
            ServiceError::PasswordHashError(_error) => Self::INTERNAL_SERVER_ERROR,
            ServiceError::LoginOccupied => Self::CONFLICT,
            ServiceError::UserWasNotFound => Self::NOT_FOUND,
        }
    }
}
