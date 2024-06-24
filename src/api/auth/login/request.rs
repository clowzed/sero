use std::fmt::{self, Debug};

use serde::{Deserialize, Serialize};
use utoipa::{schema, ToSchema};
use validator::{Validate, ValidationError};

#[derive(Deserialize, Serialize, Validate, ToSchema)]
pub struct LoginRequest {
    /// The username used for authentication.
    /// It must adhere to the following criteria:
    /// - It can contain letters (a-z), numbers (0-9), and periods (.).
    /// - It cannot contain any of the following characters: & =  ' - + , < >
    /// - It cannot have multiple periods (.) consecutively.
    /// - Minimum length of 5 characters.
    /// - Maximum length of 40 characters.
    #[validate(
        length(min = 5, max = 40),
        custom(
        function = validate_login,
        message = "Login can contain letters (a-z), numbers (0-9), and periods (.),
                   and cannot contain any of the following characters: & = ' + , < > or multiple periods (.)"
    ))]
    #[schema(min_length = 5, max_length = 40)]
    pub login: String,

    /// The password used for authentication.
    /// It must meet the following requirements:
    /// - Minimum length of 12 characters.
    /// - Maximum length of 40 characters.
    /// - A combination of letters, numbers, and symbols.
    #[validate(
        length(min = 12, max = 40),
        custom(function = validate_password,
        message = "Minimum length of 12 characters and maximum length of 40 characters and a combination of
                   letters, numbers, and symbols.")
    )]
    #[schema(min_length = 12, max_length = 40)]
    pub password: String,
}

fn validate_login(login: &str) -> Result<(), ValidationError> {
    let invalid_chars = "&='+,<>";
    let invalid_double_period = "..";

    if login.chars().any(|c| invalid_chars.contains(c)) || login.contains(invalid_double_period) {
        return Err(ValidationError::new(
            "Rules for login: Login can contain letters (a-z), numbers (0-9), and periods (.),
        and cannot contain any of the following characters: & = ' + , < > or multiple periods (.)",
        ));
    }

    Ok(())
}

fn validate_password(password: &str) -> Result<(), ValidationError> {
    let mut has_digit = false;
    let mut has_special_char = false;

    for c in password.chars() {
        if !has_digit && c.is_ascii_digit() {
            has_digit = true;
        } else if c.is_ascii_punctuation() || c.is_ascii_whitespace() {
            has_special_char = true;
        }

        if has_digit && has_special_char {
            return Ok(());
        }
    }

    Err(ValidationError::new(
        "Rules for password: Minimum length of 12 characters and maximum length of 40 characters and a combination of
        letters, numbers, and symbols.",
    ))
}

impl Debug for LoginRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoginRequest")
            .field("login", &self.login)
            .field("password", &"***")
            .finish()
    }
}
