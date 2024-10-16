use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error")]
    Database(#[from] sqlx::Error),
    #[error("Not found")]
    NotFound,
    #[error("Bad request")]
    BadRequest,
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
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "something went wrong"),
            AppError::NotFound => (StatusCode::NOT_FOUND, "not found"),
            AppError::BadRequest => (StatusCode::BAD_REQUEST, "bad request"),
        };

        let body = Json(json!({
            "status": message
        }));

        (status, body).into_response()
    }
}
