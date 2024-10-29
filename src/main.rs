use std::time::Duration;

use axum::Router;
use config::{state::AppState, ENV};
use tokio::{net::TcpListener, signal};
use tower::ServiceBuilder;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

mod config;
mod database;
mod error;
mod model;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = AppState::new().await;

    let app = Router::new()
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::new(Duration::from_secs(10))),
        )
        .with_state(state.clone());

    axum::serve(
        TcpListener::bind(format!("0.0.0.0:{}", &ENV.port))
            .await
            .unwrap(),
        app,
    )
    .with_graceful_shutdown(shutdown(state))
    .await
    .unwrap();
    Ok(())
}

pub async fn shutdown(state: AppState) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to listen for the Ctrl+C signal");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to listen for the SIGTERM signal")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    };

    state.db.close().await;
}
