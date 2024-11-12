use crate::{config::state::AppState, database, error::AppError, model::user::CreateUserReq};
use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;
use validator::Validate;

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserReq>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    let password = bcrypt::hash(payload.password, bcrypt::DEFAULT_COST).map_err(|err| {
        AppError::Other(
            anyhow::Error::new(err)
                .context("Failed to hash the password")
                .context(format!("Failed to create user: {}", payload.email)),
        )
    })?;

    database::user::create(payload.email, payload.name, password, &state.db)
        .await
        .map_err(AppError::from_db_error)?;

    Ok(Json(json!({
        "status": "ok",
    })))
}
