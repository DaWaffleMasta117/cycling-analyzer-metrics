use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use crate::{
    db::get_rides_for_athlete,
    models::{PowerCurveQuery, PowerCurveResponse},
    power_curve::calculate_power_curve,
};

// Shared database connection wrapped in a thread-safe mutex
pub type DbState = Arc<Mutex<Connection>>;

pub async fn health() -> &'static str {
    "ok"
}

pub async fn get_power_curve(
    State(db):   State<DbState>,
    Query(query): Query<PowerCurveQuery>,
) -> Result<Json<PowerCurveResponse>, StatusCode> {
    let conn = db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rides = get_rides_for_athlete(
        &conn,
        query.athlete_id,
        query.from.as_deref(),
        query.to.as_deref(),
    )
    .map_err(|e| {
        tracing::error!("DB error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if rides.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Use the most recent ride's weight as the current weight
    let weight_kg = rides.first()
        .map(|r| r.weight_kg_at_time)
        .unwrap_or(0.0);

    let curve = calculate_power_curve(&rides, weight_kg);

    Ok(Json(PowerCurveResponse {
        athlete_id: query.athlete_id,
        weight_kg,
        curve,
        generated_at: chrono::Utc::now().to_rfc3339(),
    }))
}