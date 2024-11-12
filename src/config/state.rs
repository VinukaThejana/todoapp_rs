use super::ENV;
use log::error;
use redis::{aio::MultiplexedConnection, Client as RedisClient, RedisError};
use sea_orm::{Database, DatabaseConnection};

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub rd: RedisClient,
}

impl AppState {
    pub async fn new() -> Self {
        let db: DatabaseConnection =
            Database::connect(&*ENV.database_url)
                .await
                .unwrap_or_else(|_| {
                    error!("Failed to connect to the database, please check the connection URI");
                    std::process::exit(1);
                });

        let rd = RedisClient::open(&*ENV.redis_url).unwrap_or_else(|err| {
            error!("{}", err);
            std::process::exit(1);
        });

        Self { db, rd }
    }
}

impl AppState {
    pub async fn get_redis_conn<E>(&self) -> Result<MultiplexedConnection, E>
    where
        RedisError: Into<E>,
    {
        self.rd
            .get_multiplexed_async_connection()
            .await
            .map_err(Into::into)
    }
}
