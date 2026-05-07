mod db;
mod handlers;
mod models;
mod power_curve;

use std::sync::{Arc, Mutex};
use axum::{routing::get, Router};
use rusqlite::Connection;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // The database path must be set via the DB_PATH environment variable on the
    // server. In development it falls back to the relative path used when running
    // both services side-by-side from the repo root.
    //
    // On the server, set it like:
    //   export DB_PATH=/opt/cycling-analyzer/cycling-analyzer.db
    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| {
        let default = "../cycling-analyzer-api/CyclingAnalyzer.Api/cycling-analyzer.db"
            .to_string();
        tracing::warn!(
            "DB_PATH env var not set — falling back to relative dev path: {}. \
             This will fail on a server. Set DB_PATH to the absolute path of \
             cycling-analyzer.db before deploying.",
            default
        );
        default
    });

    tracing::info!("Connecting to database at {db_path}");

    let conn  = Connection::open(&db_path)?;
    let state = Arc::new(Mutex::new(conn));

    // CORS — allow requests from the React dev server
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/power-curve", get(handlers::get_power_curve))
        .route("/ride-stats", get(handlers::get_ride_stats_handler))
        .layer(cors)
        .with_state(state);

    let addr = "0.0.0.0:3001";
    tracing::info!("Metrics engine listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}