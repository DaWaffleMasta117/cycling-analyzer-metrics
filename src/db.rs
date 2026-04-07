use anyhow::Result;
use rusqlite::{Connection, params};
use crate::models::Ride;

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