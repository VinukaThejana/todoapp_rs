use anyhow::Error as AnyhowError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("failed to create the token : {0}")]
    Creation(#[source] AnyhowError),

    #[error("failed to validate the token : {0}")]
    Validation(#[source] AnyhowError),

    #[error("failed to parse the token : {0}")]
    Parsing(#[source] AnyhowError),

    #[error("invalid token fromat provided : {0}")]
    InvalidFormat(#[source] AnyhowError),

    #[error("token has some missing claims : {0}")]
    MissingClaims(#[source] AnyhowError),

    #[error("other error : {0}")]
    Other(#[source] AnyhowError),
}
