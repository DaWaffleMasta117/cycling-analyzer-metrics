use serde::{Deserialize, Serialize};

// Matches the Rides table the .NET API created
#[derive(Debug)]
pub struct Ride {
    pub id:                    i64,
    pub athlete_id:            i64,
    pub moving_time_seconds:   i32,
    pub average_power_watts:   f32,
    pub max_power_watts:       f32,
    pub start_date:            String,
    pub weight_kg_at_time:     f32,
}

// A single point on the power curve
// duration_seconds → best average watts over that duration
#[derive(Debug, Serialize, Deserialize)]
pub struct PowerCurvePoint {
    pub duration_seconds: u32,
    pub watts:            f32,
    pub watts_per_kg:     f32,
}

// The full power curve response sent back to .NET
#[derive(Debug, Serialize, Deserialize)]
pub struct PowerCurveResponse {
    pub athlete_id:   i64,
    pub weight_kg:    f32,
    pub curve:        Vec<PowerCurvePoint>,
    pub generated_at: String,
}

// Query params for the power curve endpoint
#[derive(Debug, Deserialize)]
pub struct PowerCurveQuery {
    pub athlete_id: i64,
    pub from:       Option<String>,
    pub to:         Option<String>,
}