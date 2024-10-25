use std::net::SocketAddr;

use axum::{routing::post, Router};
use config::{AppState, Config};
use handlers::todo::create_todo;
use tokio::net::TcpListener;

mod config;
mod db;
mod error;
mod handlers;
mod models;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new()
        .route("/todos", post(create_todo))
        .with_state(AppState::new(Config::new()).await);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let tcp = TcpListener::bind(&addr).await?;

    axum::serve(tcp, app).await?;
    Ok(())
}
