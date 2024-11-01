use axum::{http::StatusCode, response::IntoResponse, Json};
use sea_orm::{DbErr, RuntimeErr};
use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Database(#[from] DbErr),

    #[error("not found")]
    NotFound,

    #[error("bad request")]
    BadRequest,

    #[error("unique violation")]
    UniqueViolation,

    #[error(transparent)]
    Validation(#[from] ValidationErrors),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl AppError {
    pub fn from_db_error(err: DbErr) -> Self {
        match err {
            DbErr::RecordNotFound(_) => AppError::NotFound,
            err => {
                if is_unique_violation(&err) {
                    AppError::UniqueViolation
                } else {
                    AppError::Database(err)
                }
            }
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
            AppError::NotFound => {
                log::error!("Not found");
                (StatusCode::NOT_FOUND, String::from("not found"))
            }
            AppError::BadRequest => {
                log::error!("Bad request");
                (StatusCode::BAD_REQUEST, String::from("bad request"))
            }
            AppError::UniqueViolation => {
                log::error!("Unique violation");
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
                            .unwrap_or_else(|| "invalid  input".to_string())
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
