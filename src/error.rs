use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error("not found")]
    NotFound,

    #[error("bad request")]
    BadRequest,

    #[error(transparent)]
    Validation(#[from] ValidationErrors),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl AppError {
    pub fn from_sqlx_error(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound,
            _ => AppError::Database(err),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("something went wrong"),
            ),
            AppError::NotFound => (StatusCode::NOT_FOUND, String::from("not found")),
            AppError::BadRequest => (StatusCode::BAD_REQUEST, String::from("bad request")),
            AppError::Validation(validation_errors) => {
                let message = validation_errors
                    .field_errors()
                    .iter()
                    .map(|(field, err)| {
                        format!(
                            "{}: {}",
                            field,
                            err.first()
                                .and_then(|e| e.message.as_ref())
                                .map(|msg| msg.to_string())
                                .unwrap_or_else(|| "invalid  input".to_string())
                        )
                    })
                    .collect::<Vec<String>>()
                    .join(", ");

                (StatusCode::BAD_REQUEST, message)
            }
            AppError::Other(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("something went wrong"),
            ),
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
