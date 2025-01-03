use crate::token::error::TokenError;
use anyhow::{anyhow, Error as AnyhowError};
use axum::{http::StatusCode, response::IntoResponse, Json};
use redis::RedisError;
use sea_orm::{DbErr, RuntimeErr};
use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Database(#[from] DbErr),

    #[error(transparent)]
    Redis(#[from] RedisError),

    #[error("(NotFound): {0}")]
    NotFound(#[source] AnyhowError),

    #[error("(BadRequest): {0}")]
    BadRequest(#[source] AnyhowError),

    #[error("(UniqueViolation): {0}")]
    UniqueViolation(#[source] AnyhowError),

    #[error("(Unauthorized): {0}")]
    Unauthorized(#[source] AnyhowError),

    #[error("(IncorrectCredentials): {0}")]
    IncorrectCredentials(#[source] AnyhowError),

    #[error(transparent)]
    Validation(#[from] ValidationErrors),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl AppError {
    pub fn from_db_error(err: DbErr) -> Self {
        match err {
            DbErr::RecordNotFound(err) => AppError::NotFound(anyhow!(err)),
            err => {
                if is_unique_violation(&err) {
                    AppError::UniqueViolation(err.into())
                } else {
                    AppError::Database(err)
                }
            }
        }
    }

    pub fn from_token_error(err: TokenError) -> Self {
        match err {
            TokenError::MissingClaims(source)
            | TokenError::InvalidFormat(source)
            | TokenError::Parsing(source)
            | TokenError::Validation(source) => AppError::Unauthorized(source.into()),

            _ => AppError::Other(err.into()),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::Database(err) => {
                log::error!("Database error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    String::from("something went wrong"),
                )
            }
            AppError::Redis(err) => {
                log::error!("Redis error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    String::from("something went wrong"),
                )
            }
            AppError::NotFound(err) => {
                log::error!("{err}");
                (StatusCode::NOT_FOUND, String::from("not found"))
            }
            AppError::BadRequest(err) => {
                log::error!("{err}");
                (StatusCode::BAD_REQUEST, String::from("bad request"))
            }
            AppError::Unauthorized(err) => {
                log::error!("{err}");
                (StatusCode::UNAUTHORIZED, String::from("unauthorized"))
            }
            AppError::IncorrectCredentials(err) => {
                log::error!("{err}");
                (
                    StatusCode::UNAUTHORIZED,
                    String::from("credentials are not valid"),
                )
            }
            AppError::UniqueViolation(err) => {
                log::error!("{err}");
                (StatusCode::CONFLICT, String::from("already exists"))
            }
            AppError::Validation(validation_errors) => {
                let message = validation_errors
                    .field_errors()
                    .values()
                    .map(|err| {
                        err.first()
                            .and_then(|e| e.message.as_ref())
                            .map(|msg| msg.to_string())
                            .unwrap_or_else(|| String::from("invalid input"))
                    })
                    .collect::<Vec<String>>()
                    .first()
                    .unwrap_or(&String::from("invalid input"))
                    .to_string();

                log::error!("Validation error: {}", message);
                (StatusCode::BAD_REQUEST, message)
            }
            AppError::Other(err) => {
                log::error!("Internal server error : {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    String::from("something went wrong"),
                )
            }
        };

        (
            status,
            Json(json!({
                "status": message
            })),
        )
            .into_response()
    }
}

fn is_unique_violation(err: &DbErr) -> bool {
    match err {
        DbErr::Query(RuntimeErr::SqlxError(error)) => {
            if let Some(db_error) = error.as_database_error() {
                if let Some(code) = db_error.code() {
                    code.as_ref() == "23505"
                } else {
                    false
                }
            } else {
                false
            }
        }
        _ => false,
    }
}
