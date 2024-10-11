mod config;

use config::{AppState, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app_state = AppState::new(Config::new()).await;

    Ok(())
}
