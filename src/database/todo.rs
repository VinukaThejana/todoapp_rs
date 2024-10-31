use crate::{
    entity::{prelude::Todo, session::Model, todo},
    model::todo::{PaginatedTodo, UpdateTodo},
};
use sea_orm::*;
use todoapp_rs::Paginator;

pub async fn create(data: todo::Model, db: &DatabaseConnection) -> Result<todo::Model, DbErr> {
    Todo::insert(todo::ActiveModel {
        title: Set(data.title),
        user_id: Set(data.user_id),
        content: Set(data.content),
        completed: Set(data.completed),
        ..Default::default()
    })
    .exec_with_returning(db)
    .await
}

pub async fn find_by_id(id: String, db: &DatabaseConnection) -> Result<Option<todo::Model>, DbErr> {
    Todo::find_by_id(id).one(db).await
}

pub async fn find_by_user_id(
    user_id: String,
    paginator: Paginator,
    db: &DatabaseConnection,
) -> Result<PaginatedTodo, DbErr> {
    let mut result = PaginatedTodo::default();
    let mut todos = Todo::find()
        .filter(todo::Column::UserId.eq(user_id))
        .offset(paginator.skip)
        .limit(paginator.take + 1)
        .all(db)
        .await?;

    if todos.len().try_into().unwrap_or(0) == paginator.take + 1 {
        result.has_next = true;
        result.next_offset = Some(paginator.skip + paginator.take + 1);
    }
    todos.pop();
    result.todos = todos;

    Ok(result)
}

pub async fn mark_complete(
    id: String,
    user_id: String,
    is_complete: bool,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    let result = todo::ActiveModel {
        id: Set(id),
        user_id: Set(user_id),
        completed: Set(is_complete),
        ..Default::default()
    };

    result.save(db).await?;

    Ok(())
}

pub async fn update(data: UpdateTodo, db: &DatabaseConnection) -> Result<todo::Model, DbErr> {
    let mut result = todo::ActiveModel {
        id: Set(data.id),
        user_id: Set(data.user_id),
        ..Default::default()
    };

    if let Some(title) = data.title {
        result.title = Set(title);
    }
    if let Some(content) = data.content {
        result.content = Set(content);
    }
    if let Some(completed) = data.completed {
        result.completed = Set(completed);
    }

    result.save(db).await?.try_into_model()
}

pub async fn delete(id: String, user_id: String, db: &DatabaseConnection) -> Result<(), DbErr> {
    todo::ActiveModel {
        id: Set(id),
        user_id: Set(user_id),
        ..Default::default()
    }
    .delete(db)
    .await?;

    Ok(())
}
