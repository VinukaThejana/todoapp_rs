use crate::entity::todo;

#[derive(Default)]
pub struct PaginatedTodo {
    pub todos: Vec<todo::Model>,
    pub next_offset: Option<u64>,
    pub has_next: bool,
}

pub struct UpdateTodo {
    pub id: String,
    pub user_id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub completed: Option<bool>,
}
