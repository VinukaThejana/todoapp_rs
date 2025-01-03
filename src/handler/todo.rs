use crate::{
    config::state::AppState,
    database,
    error::AppError,
    model::todo::{CreateTodoReq, TodoIDReq, UpdateTodoReq},
    utils::paginate::Paginator,
};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Extension, Json,
};
use serde::Deserialize;
use serde_json::json;
use validator::Validate;

pub async fn create(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Json(payload): Json<CreateTodoReq>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    database::todo::create(user_id, payload, &state.db)
        .await
        .map_err(AppError::from_db_error)?;

    Ok(Json(json!({
        "status": "ok"
    })))
}

#[derive(Deserialize, Default)]
pub struct Pagination {
    page: Option<u64>,
    limit: Option<u64>,
}

pub async fn list(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Query(query): Query<Pagination>,
) -> Result<impl IntoResponse, AppError> {
    let take = query.limit.unwrap_or(5);
    let skip = (query.page.unwrap_or(1) - 1) * take;

    let todos = database::todo::find_by_user_id(user_id, Paginator { skip, take }, &state.db)
        .await
        .map_err(AppError::from_db_error)?;

    Ok(Json(todos))
}

pub async fn update(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Json(payload): Json<UpdateTodoReq>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    database::todo::update(user_id, payload, &state.db)
        .await
        .map_err(AppError::from_db_error)?;

    Ok(Json(json!({
        "status": "ok"
    })))
}

pub async fn mark(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Json(payload): Json<TodoIDReq>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    database::todo::mark(payload.id, user_id, &state.db)
        .await
        .map_err(AppError::from_db_error)?;

    Ok(Json(json!({
        "status": "ok"
    })))
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Json(payload): Json<TodoIDReq>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    database::todo::delete(payload.id, user_id, &state.db)
        .await
        .map_err(AppError::from_db_error)?;

    Ok(Json(json!({
        "status": "ok"
    })))
}
