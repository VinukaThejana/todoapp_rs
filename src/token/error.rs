use thiserror::Error;

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("failed to create the token: {0}")]
    Creation(String),

    #[error("faild to validate the token: {0}")]
    Validation(String),

    #[error("failed to parse the token: {0}")]
    Parsing(String),

    #[error("failed to load the secret keys: {0}")]
    SecretKey(String),

    #[error("invalid token format: {0}")]
    InvalidFormat(String),

    #[error("missing required claim: {0}")]
    MissingClaim(String),

    #[error("other error: {0}")]
    Other(String),
}
