use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use crate::{
    db::{get_rides_for_athlete, get_power_streams_for_rides, get_athlete_weight},
    models::{PowerCurveQuery, PowerCurveResponse},
    power_curve::calculate_power_curve,
};

// Shared database connection wrapped in a thread-safe mutex
pub type DbState = Arc<Mutex<Connection>>;

pub async fn health() -> &'static str {
    "ok"
}

pub async fn get_power_curve(
    State(db):    State<DbState>,
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
        tracing::error!("DB error fetching rides: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if rides.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Use the athlete's current weight from their profile (updated on every login),
    // not the historical weight stored on individual rides.
    let weight_kg = get_athlete_weight(&conn, query.athlete_id)
        .unwrap_or(0.0);

    // Fetch the second-by-second power stream for every ride in one query
    let ride_ids: Vec<i64> = rides.iter().map(|r| r.id).collect();
    let streams = get_power_streams_for_rides(&conn, &ride_ids)
        .map_err(|e| {
            tracing::error!("DB error fetching power streams: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let curve = calculate_power_curve(&rides, &streams, weight_kg);

    Ok(Json(PowerCurveResponse {
        athlete_id: query.athlete_id,
        weight_kg,
        curve,
        generated_at: chrono::Utc::now().to_rfc3339(),
    }))
}
