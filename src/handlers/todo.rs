use axum::{extract::State, Json};
use validator::Validate;

use crate::{
    config::AppState,
    db,
    error::AppError,
    models::todo::{CreateTodoReq, Todo},
};

pub async fn create_todo(
    State(state): State<AppState>,
    Json(todo): Json<CreateTodoReq>,
) -> Result<Json<Todo>, AppError> {
    todo.validate()?;

    let todo = db::todo::create(&state.db, Todo::from(todo)).await?;
    Ok(Json(todo))
}
