use crate::database;
use crate::{config::state::AppState, error::AppError};
use anyhow::anyhow;
use axum::Json;
use axum::{extract::State, response::IntoResponse, Extension};
use serde_json::json;
use urlencoding::encode;

pub async fn profile(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
) -> Result<impl IntoResponse, AppError> {
    let user = database::user::find_by_id(user_id, &state.db)
        .await
        .map_err(AppError::from_db_error)?
        .ok_or_else(|| AppError::Unauthorized(anyhow!("Cannot find a user with the given ID")))?;

    Ok(Json(json!({
        "user": {
            "id": user.id,
            "email": user.email,
            "name": user.name,
            "photo_url": format!("https://api.dicebear.com/9.x/notionists/svg?seed={}", encode(&user.name))
        },
    })))
}
