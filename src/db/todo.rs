use sqlx::PgPool;
use validator::ValidateRequired;

use crate::{
    error::AppError,
    models::todo::{Todo, UpdateTodoReq},
};

pub async fn create(pool: &PgPool, todo: Todo) -> Result<Todo, AppError> {
    sqlx::query_as!(
        Todo,
        r#"
        INSERT INTO todo (title, completed, description)
        VALUES ($1, $2, $3)
        RETURNING id, title, completed, description
        "#,
        todo.title,
        todo.completed,
        todo.description,
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::from_sqlx_error)
}

pub async fn get(pool: &PgPool) -> Result<Vec<Todo>, AppError> {
    sqlx::query_as!(
        Todo,
        r#"
        SELECT id, title, completed, description FROM todo
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from_sqlx_error)
}

pub async fn update(pool: &PgPool, id: i32, todo: UpdateTodoReq) -> Result<(), AppError> {
    Ok(())
}
