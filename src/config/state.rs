use sqlx::PgPool;

use super::ENV;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

impl AppState {
    pub async fn new() -> Self {
        let db = PgPool::connect(&ENV.database_url)
            .await
            .expect("Failed to connect to the database");

        Self { db }
    }
}
