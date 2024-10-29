use sqlx::PgPool;

use super::ENV;
use log::error;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

impl AppState {
    pub async fn new() -> Self {
        let db = PgPool::connect(&ENV.database_url)
            .await
            .unwrap_or_else(|_| {
                error!("Failed to connect to the database, please check the connection URI");
                std::process::exit(1);
            });

        Self { db }
    }
}
