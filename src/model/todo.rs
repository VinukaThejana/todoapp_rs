use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Todo {
    id: String,
    user_id: String,
    title: String,
    content: String,
    completed: bool,
    created_at: u64,
    updated_at: u64,
}
