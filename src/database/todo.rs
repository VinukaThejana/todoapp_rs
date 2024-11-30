use crate::{
    entity::{
        prelude::Todo,
        todo::{self},
    },
    model::todo::{CreateTodoReq, PaginatedTodo, UpdateTodoReq},
    utils::paginate::Paginator,
};
use sea_orm::*;

pub async fn create(
    user_id: String,
    data: CreateTodoReq,
    db: &DatabaseConnection,
) -> Result<todo::Model, DbErr> {
    Todo::insert(todo::ActiveModel {
        title: Set(data.title),
        user_id: Set(user_id),
        content: Set(data.content),
        completed: Set(false),
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
        todos.pop();
    }
    result.todos = todos;

    Ok(result)
}

#[derive(DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "Todo")]
struct TodoCompleted {
    completed: bool,
}

pub async fn mark(id: String, user_id: String, db: &DatabaseConnection) -> Result<(), DbErr> {
    let completed = Todo::find_by_id(id.clone())
        .into_partial_model::<TodoCompleted>()
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound(String::from(
            "Todo not found for the given id",
        )))?
        .completed;

    let result = todo::ActiveModel {
        id: Set(id),
        user_id: Set(user_id),
        completed: Set(!completed),
        ..Default::default()
    };

    result.save(db).await?;
    Ok(())
}

pub async fn update(
    user_id: String,
    data: UpdateTodoReq,
    db: &DatabaseConnection,
) -> Result<todo::Model, DbErr> {
    let mut result = todo::ActiveModel {
        id: Set(data.id),
        user_id: Set(user_id),
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
