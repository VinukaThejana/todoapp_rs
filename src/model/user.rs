use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use validator::{Validate, ValidationError};

pub struct UpdateUser {
    pub id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDetails {
    pub email: String,
    pub name: String,
    pub photo_url: String,
}

#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct CreateUserReq {
    #[validate(email(message = "please provide a valid email address"))]
    pub email: String,

    #[validate(length(
        min = 3,
        max = 100,
        message = "name must be between 3 and 100 characters"
    ))]
    pub name: String,

    #[validate(custom(function = "validate_password"))]
    pub password: String,
}

#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct LoginUserReq {
    #[validate(email(message = "email address is not valid"))]
    pub email: String,

    #[validate(custom(function = "validate_password"))]
    pub password: String,
}

fn validate_password(password: &str) -> Result<(), ValidationError> {
    let checks = [
        (
            password.len() < 8,
            "password must be have more than 8 characters",
        ),
        (
            password.len() > 200,
            "password must be less than 200 characters",
        ),
        (
            password.chars().all(|c| c.is_ascii_alphanumeric()),
            "password must contain at least one special character",
        ),
        (
            !password.chars().any(|c| c.is_lowercase()),
            "password must contain at least one lowercae character",
        ),
        (
            !password.chars().any(|c| c.is_uppercase()),
            "password must contain at least one uppercase character",
        ),
        (
            !password.chars().any(|c| c.is_numeric()),
            "password must contain at least one numeric character",
        ),
    ];

    for (not_valid, err_message) in checks {
        if not_valid {
            return Err(
                ValidationError::new("password").with_message(Cow::Owned(err_message.to_string()))
            );
        }
    }

    Ok(())
}
