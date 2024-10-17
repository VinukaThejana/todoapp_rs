use dotenv::dotenv;
use sqlx::PgPool;

pub struct Config {
    pub database_url: String,
}

impl Config {
    pub fn new() -> Config {
        dotenv().unwrap();

        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        Config { database_url }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

impl AppState {
    pub async fn new(config: Config) -> Self {
        let db = PgPool::connect(&config.database_url)
            .await
            .expect("Failed to connect to database");

        AppState { db }
    }
}
