use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Not found")]
    NotFound,

    #[error("Bad request")]
    BadRequest,

    #[error("Validation error")]
    Validation(#[from] ValidationErrors),
}

impl AppError {
    pub fn from_sqlx_error(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => Self::NotFound,
            _ => Self::Database(error),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("database error"),
            ),
            AppError::NotFound => (StatusCode::NOT_FOUND, String::from("not found")),
            AppError::BadRequest => (StatusCode::BAD_REQUEST, String::from("bad request")),
            AppError::Validation(errs) => {
                let message = errs
                    .field_errors()
                    .iter()
                    .map(|(field, errors)| {
                        format!(
                            "{}: {}",
                            field,
                            errors
                                .first()
                                .and_then(|error| error.message.as_ref())
                                .map(|msg| msg.to_string())
                                .unwrap_or_else(|| "invalid input".to_string())
                        )
                    })
                    .collect::<Vec<String>>()
                    .join(", ");

                (StatusCode::BAD_REQUEST, message)
            }
        };

        let body = Json(json!({
            "status": message
        }));

        (status, body).into_response()
    }
}
