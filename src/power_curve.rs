use crate::models::{PowerCurvePoint, Ride};

// Standard durations that match the frontend constants exactly
pub const DURATIONS: &[u32] = &[
    1, 5, 10, 30, 60, 120, 300, 600, 1200, 1800, 3600, 7200, 10800, 21600,
];

pub fn calculate_power_curve(rides: &[Ride], weight_kg: f32) -> Vec<PowerCurvePoint> {
    DURATIONS
        .iter()
        .map(|&duration| {
            let best_watts = best_average_power(rides, duration);
            PowerCurvePoint {
                duration_seconds: duration,
                watts:            best_watts,
                watts_per_kg:     if weight_kg > 0.0 {
                                      (best_watts / weight_kg * 100.0).round() / 100.0
                                  } else {
                                      0.0
                                  },
            }
        })
        .collect()
}

// Finds the best average power across all rides for a given duration
fn best_average_power(rides: &[Ride], duration_seconds: u32) -> f32 {
    rides
        .iter()
        .filter_map(|ride| estimate_power_for_duration(ride, duration_seconds))
        .fold(0.0_f32, f32::max)
}

// Estimates best power for a duration from a ride's summary data
//
// NOTE: This is an estimation from summary data. When we have access
// to raw power streams from the Strava API we can replace this with
// an exact calculation using a sliding window over the data points.
fn estimate_power_for_duration(ride: &Ride, duration_seconds: u32) -> Option<f32> {
    let ride_duration = ride.moving_time_seconds as u32;

    // Skip rides shorter than the duration we're looking for
    if ride_duration < duration_seconds { return None; }

    // Skip rides with no power data
    if ride.average_power_watts <= 0.0 { return None; }

    // For durations shorter than the ride we use the critical power model
    // to estimate what the athlete could sustain for that duration.
    // Formula: P(t) = avg_power * (ride_duration / duration)^0.07
    // The 0.07 exponent is a well established empirical constant for
    // the power-duration relationship in cycling.
    let ratio = ride_duration as f32 / duration_seconds as f32;
    let watts = ride.average_power_watts * ratio.powf(0.07);

    Some((watts * 10.0).round() / 10.0) // round to 1 decimal place
}