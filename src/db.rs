use anyhow::Result;
use rusqlite::{Connection, params};
use crate::models::Ride;

pub fn get_rides_for_athlete(
    conn: &Connection,
    athlete_id: i64,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<Vec<Ride>> {
    let mut query = String::from(
        "SELECT
            id,
            athlete_id,
            moving_time_seconds,
            average_power_watts,
            max_power_watts,
            start_date,
            weight_kg_at_time
         FROM Rides
         WHERE athlete_id = ?1"
    );

    // Append date filters if provided
    if from.is_some() { query.push_str(" AND start_date >= ?2"); }
    if to.is_some() { query.push_str(" AND start_date <= ?3"); }
    query.push_str(" ORDER BY start_date DESC");

    let mut stmt = conn.prepare(&query)?;

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