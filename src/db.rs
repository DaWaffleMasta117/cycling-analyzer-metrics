use anyhow::Result;
use rusqlite::{Connection, params};
use std::collections::HashMap;
use crate::models::Ride;

/// Returns the athlete's current weight from the Athletes table.
/// This is updated on every Strava login, so it always reflects the
/// most recent weight on their Strava profile.
pub fn get_athlete_weight(conn: &Connection, athlete_id: i64) -> Result<f32> {
    let weight = conn.query_row(
        "SELECT WeightKg FROM Athletes WHERE Id = ?1",
        params![athlete_id],
        |row| row.get::<_, f32>(0),
    )?;
    Ok(weight)
}

pub fn get_rides_for_athlete(
    conn: &Connection,
    athlete_id: i64,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<Vec<Ride>> {
    let query =
        "SELECT
            Id,
            AthleteId,
            MovingTimeSeconds,
            AveragePowerWatts,
            MaxPowerWatts,
            StartDate,
            WeightKgAtTime
         FROM Rides
         WHERE AthleteId = ?1
           AND (?2 IS NULL OR StartDate >= ?2)
           AND (?3 IS NULL OR StartDate <= ?3)
         ORDER BY StartDate DESC";

    let mut stmt = conn.prepare(query)?;

    let rides = stmt.query_map(
        params![athlete_id, from, to],
        |row| {
            Ok(Ride {
                id: row.get(0)?,
                athlete_id: row.get(1)?,
                moving_time_seconds: row.get(2)?,
                average_power_watts: row.get(3)?,
                max_power_watts: row.get(4)?,
                start_date: row.get(5)?,
                weight_kg_at_time: row.get(6)?,
            })
        },
    )?
    .filter_map(|r| r.ok())
    .collect();

    Ok(rides)
}

/// Fetches the second-by-second watts stream for each of the given ride IDs.
///
/// Returns a map of ride_id → watts array where each element is the power in
/// watts for that second of the ride (None = no reading recorded).
pub fn get_power_streams_for_rides(
    conn: &Connection,
    ride_ids: &[i64],
) -> Result<HashMap<i64, Vec<Option<i32>>>> {
    if ride_ids.is_empty() {
        return Ok(HashMap::new());
    }

    // Build a parameterised IN clause: (?, ?, ?, …)
    let placeholders = std::iter::repeat("?")
        .take(ride_ids.len())
        .collect::<Vec<_>>()
        .join(", ");

    let query = format!(
        "SELECT RideId, WattsJson FROM RidePowerStreams WHERE RideId IN ({placeholders})"
    );

    let mut stmt = conn.prepare(&query)?;

    let mut map = HashMap::new();

    let rows = stmt.query_map(
        rusqlite::params_from_iter(ride_ids.iter()),
        |row| {
            let ride_id: i64   = row.get(0)?;
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
