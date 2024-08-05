use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};
use utoipa::{schema, ToSchema};
use validator::Validate;

#[derive(Deserialize, Serialize, Validate, ToSchema)]
pub struct LoginRequest {
    /// The username used for authentication.
    /// It must adhere to the following criteria:
    /// - Minimum length of 5 characters.
    /// - Maximum length of 40 characters.
    #[validate(length(min = 5, max = 40))]
    #[schema(min_length = 5, max_length = 40)]
    pub login: String,

    /// The password used for authentication.
    /// It must meet the following requirements:
    /// - Minimum length of 12 characters.
    /// - Maximum length of 40 characters.
    #[validate(length(min = 12, max = 40))]
    #[schema(min_length = 12, max_length = 40)]
    pub password: String,
}

impl Debug for LoginRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoginRequest")
            .field("login", &self.login)
            .field("password", &"***")
            .finish()
    }
}
