use serde::{Deserialize, Serialize};

// ─── Power curve ──────────────────────────────────────────────────────────────

/// Matches the Rides table columns needed for MMP curve calculation.
/// weight_kg_at_time was removed from the DB — athlete weight is now read
/// from the Athletes table via get_athlete_weight().
#[derive(Debug)]
pub struct Ride {
    pub id:                  i64,
    pub athlete_id:          i64,
    pub moving_time_seconds: i32,
    pub average_power_watts: f32,
    pub max_power_watts:     f32,
    pub start_date:          String,
}

/// A single point on the Mean Maximal Power curve.
#[derive(Debug, Serialize, Deserialize)]
pub struct PowerCurvePoint {
    pub duration_seconds: u32,
    pub watts:            f32,
    pub watts_per_kg:     f32,
}

/// Full response from GET /power-curve.
#[derive(Debug, Serialize, Deserialize)]
pub struct PowerCurveResponse {
    pub athlete_id:   i64,
    pub weight_kg:    f32,
    pub curve:        Vec<PowerCurvePoint>,
    pub generated_at: String,
}

/// Query params for GET /power-curve.
#[derive(Debug, Deserialize)]
pub struct PowerCurveQuery {
    pub athlete_id: i64,
    pub from:       Option<String>,
    pub to:         Option<String>,
}

// ─── Ride stats ───────────────────────────────────────────────────────────────

/// Aggregate ride statistics for a date range, returned by GET /ride-stats.
#[derive(Debug, Serialize)]
pub struct RideStatsResponse {
    pub athlete_id:      i64,
    pub weight_kg:       f32,
    /// Highest AveragePowerWatts across all rides in the range.
    pub peak_avg_watts:  f32,
    /// Highest NormalizedPowerWatts across rides that have NP data.
    pub peak_np_watts:   f32,
    /// Mean AveragePowerWatts across all rides in the range.
    pub mean_avg_watts:  f32,
    /// Mean NormalizedPowerWatts across rides that have NP data.
    pub mean_np_watts:   f32,
}

/// Query params for GET /ride-stats (same shape as PowerCurveQuery).
#[derive(Debug, Deserialize)]
pub struct RideStatsQuery {
    pub athlete_id: i64,
    pub from:       Option<String>,
    pub to:         Option<String>,
}
