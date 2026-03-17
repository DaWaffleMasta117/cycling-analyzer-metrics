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

    // Connect to the same SQLite DB the .NET API uses
    // Adjust this path to point at your actual .db file
    let db_path = std::env::var("DB_PATH")
        .unwrap_or_else(|_| "../cycling-analyzer-api/CyclingAnalyzer.Api/cycling-analyzer.db"
        .to_string());

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
        .layer(cors)
        .with_state(state);

    let addr = "0.0.0.0:3001";
    tracing::info!("Metrics engine listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}