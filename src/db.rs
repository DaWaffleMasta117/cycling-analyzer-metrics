use anyhow::Result;
use rusqlite::{Connection, params};
use std::collections::HashMap;
use crate::models::{Ride, RideStatsResponse};

/// Returns the athlete's current weight from the Athletes table.
pub fn get_athlete_weight(conn: &Connection, athlete_id: i64) -> Result<f32> {
    let weight = conn.query_row(
        "SELECT WeightKg FROM Athletes WHERE Id = ?1",
        params![athlete_id],
        |row| row.get::<_, f32>(0),
    )?;
    Ok(weight)
}

/// Returns rides for an athlete within an optional date range.
/// Only columns needed for MMP curve calculation are selected.
pub fn get_rides_for_athlete(
    conn: &Connection,
    athlete_id: i64,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<Vec<Ride>> {
    let mut stmt = conn.prepare(
        "SELECT
            Id,
            AthleteId,
            MovingTimeSeconds,
            AveragePowerWatts,
            MaxPowerWatts,
            StartDate
         FROM Rides
         WHERE AthleteId = ?1
           AND (?2 IS NULL OR StartDate >= ?2)
           AND (?3 IS NULL OR StartDate <= ?3)
         ORDER BY StartDate DESC",
    )?;

    let rides = stmt
        .query_map(params![athlete_id, from, to], |row| {
            Ok(Ride {
                id:                  row.get(0)?,
                athlete_id:          row.get(1)?,
                moving_time_seconds: row.get(2)?,
                average_power_watts: row.get(3)?,
                max_power_watts:     row.get(4)?,
                start_date:          row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rides)
}

/// Returns aggregate power statistics for an athlete within an optional date range.
///
/// - `peak_avg_watts` / `mean_avg_watts` — based on AveragePowerWatts for all
///   rides that have power data.
/// - `peak_np_watts` / `mean_np_watts` — based only on rides where
///   NormalizedPowerWatts > 0, so rides synced before NP was added don't
///   drag the numbers down to zero.
pub fn get_ride_stats(
    conn: &Connection,
    athlete_id: i64,
    from: Option<&str>,
    to: Option<&str>,
    weight_kg: f32,
) -> Result<RideStatsResponse> {
    let row = conn.query_row(
        "SELECT
            MAX(AveragePowerWatts),
            AVG(AveragePowerWatts),
            COALESCE(MAX(CASE WHEN NormalizedPowerWatts > 0 THEN NormalizedPowerWatts END), 0.0),
            COALESCE(AVG(CASE WHEN NormalizedPowerWatts > 0 THEN NormalizedPowerWatts END), 0.0)
         FROM Rides
         WHERE AthleteId = ?1
           AND (?2 IS NULL OR StartDate >= ?2)
           AND (?3 IS NULL OR StartDate <= ?3)
           AND AveragePowerWatts > 0",
        params![athlete_id, from, to],
        |row| {
            Ok((
                row.get::<_, f64>(0).unwrap_or(0.0) as f32,
                row.get::<_, f64>(1).unwrap_or(0.0) as f32,
                row.get::<_, f64>(2).unwrap_or(0.0) as f32,
                row.get::<_, f64>(3).unwrap_or(0.0) as f32,
            ))
        },
    )?;

    Ok(RideStatsResponse {
        athlete_id,
        weight_kg,
        peak_avg_watts: row.0,
        mean_avg_watts: row.1,
        peak_np_watts:  row.2,
        mean_np_watts:  row.3,
    })
}

/// Fetches the second-by-second watts stream for each of the given ride IDs.
/// Returns a map of ride_id → watts array (None = no reading / coasting).
pub fn get_power_streams_for_rides(
    conn: &Connection,
    ride_ids: &[i64],
) -> Result<HashMap<i64, Vec<Option<i32>>>> {
    if ride_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let placeholders = std::iter::repeat("?")
        .take(ride_ids.len())
        .collect::<Vec<_>>()
        .join(", ");

    let query = format!(
        "SELECT RideId, WattsJson FROM RidePowerStreams WHERE RideId IN ({placeholders})"
    );

    let mut stmt = conn.prepare(&query)?;
    let mut map  = HashMap::new();

    let rows = stmt.query_map(
        rusqlite::params_from_iter(ride_ids.iter()),
        |row| {
            let ride_id: i64    = row.get(0)?;
            let json:    String = row.get(1)?;
            Ok((ride_id, json))
        },
    )?;

    for row in rows {
        let (ride_id, json) = row?;
        let watts: Vec<Option<i32>> = serde_json::from_str(&json).unwrap_or_default();
        map.insert(ride_id, watts);
    }

    Ok(map)
}
