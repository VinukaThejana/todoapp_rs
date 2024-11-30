use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::entity::todo;

#[derive(Default, Deserialize, Serialize)]
pub struct PaginatedTodo {
    pub todos: Vec<todo::Model>,
    pub next_offset: Option<u64>,
    pub has_next: bool,
}

#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct UpdateTodoReq {
    #[validate(length(equal = 26, message = "provide a valid todo id"))]
    pub id: String,

    #[validate(length(
        min = 3,
        max = 50,
        message = "title must be between 3 and 50 characters"
    ))]
    pub title: Option<String>,

    #[validate(length(
        min = 3,
        max = 255,
        message = "content must be between 3 and 255 characters"
    ))]
    pub content: Option<String>,

    pub completed: Option<bool>,
}

#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct CreateTodoReq {
    #[validate(length(
        min = 3,
        max = 50,
        message = "title must be between 3 and 50 characters"
    ))]
    pub title: String,

    #[validate(length(
        min = 3,
        max = 255,
        message = "content must be between 3 and 255 characters"
    ))]
    pub content: String,
}

#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct TodoIDReq {
    #[validate(length(equal = 26, message = "provide a valid todo id"))]
    pub id: String,
}
