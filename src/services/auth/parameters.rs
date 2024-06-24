use serde::{Deserialize, Serialize};

pub struct JwtGenerationParameters<T>
where
    T: AsRef<str>,
{
    pub secret: T,
    pub ttl: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    pub sub: i64,
    pub iat: i64,
    pub exp: i64,
}

pub struct UserCredentials<T>
where
    T: AsRef<str>,
{
    pub login: T,
    pub password: T,
}

pub struct JwtDecodingParameters<T>
where
    T: AsRef<str>,
{
    pub token: T,
    pub secret: T,
}
