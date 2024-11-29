use crate::database;
use crate::model::user::UpdateUserReq;
use crate::token::cookies::CookieManager;
use crate::token::traits::Token;
use crate::token::types::refresh::Refresh;
use crate::token::{constants, TokenType};
use crate::{config::state::AppState, error::AppError};
use anyhow::anyhow;
use axum::http::HeaderMap;
use axum::Json;
use axum::{extract::State, response::IntoResponse, Extension};
use serde_json::json;
use urlencoding::encode;
use validator::Validate;

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

pub async fn update(
    State(state): State<AppState>,
    Json(payload): Json<UpdateUserReq>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    database::user::update(payload, &state.db)
        .await
        .map_err(AppError::from_db_error)?;

    Ok(Json(json!({
        "status": "ok"
    })))
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let refresh_token = Refresh::default(state.clone());

    let jti = refresh_token
        .verify(
            CookieManager::get(&headers, constants::REFRESH_TOKEN_COOKIE_NAME)
                .ok_or_else(|| AppError::Unauthorized(anyhow!("Refresh token not found")))?
                .value()
                .to_owned(),
            TokenType::Refresh,
        )
        .await
        .map_err(AppError::from_token_error)?
        .jti;

    refresh_token
        .delete(&jti)
        .await
        .map_err(AppError::from_token_error)?;

    database::user::delete(user_id, &state.db)
        .await
        .map_err(AppError::from_db_error)?;

    Ok(Json(json!({
        "status": "ok"
    })))
}
