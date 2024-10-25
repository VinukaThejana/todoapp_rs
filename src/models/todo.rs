use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Todo {
    pub id: i32,
    pub title: String,
    pub completed: bool,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateTodoReq {
    #[validate(length(
        min = 3,
        max = 30,
        message = "title must be between 3 and 30 characters"
    ))]
    pub title: String,

    #[validate(length(
        min = 20,
        max = 500,
        message = "description must be between 20 and 500 characters"
    ))]
    pub description: Option<String>,

    pub completed: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateTodoReq {
    #[validate(length(
        min = 3,
        max = 30,
        message = "title must be between 3 and 30 characters"
    ))]
    pub title: Option<String>,

    #[validate(length(
        min = 20,
        max = 500,
        message = "description must be between 20 and 500 characters"
    ))]
    pub description: Option<String>,

    pub completed: Option<bool>,
}

impl From<CreateTodoReq> for Todo {
    fn from(value: CreateTodoReq) -> Self {
        Todo {
            // ID will be set by the database
            id: -1,
            title: value.title,
            completed: value.completed,
            description: value.description,
        }
    }
}
