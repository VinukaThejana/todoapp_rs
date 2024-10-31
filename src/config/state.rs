use sea_orm::{Database, DatabaseConnection};

use super::ENV;
use log::error;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}

impl AppState {
    pub async fn new() -> Self {
        let db: DatabaseConnection =
            Database::connect(&ENV.database_url)
                .await
                .unwrap_or_else(|_| {
                    error!("Failed to connect to the database, please check the connection URI");
                    std::process::exit(1);
                });

        Self { db }
    }
}
