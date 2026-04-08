use std::collections::HashMap;
use crate::models::{PowerCurvePoint, Ride};

// Standard durations that match the frontend constants exactly
pub const DURATIONS: &[u32] = &[
    3, 5, 10, 30, 60, 120, 300, 600, 1200, 1800, 3600, 7200, 10800, 21600,
];

/// Computes the Mean Maximal Power (MMP) curve from second-by-second power streams.
///
/// For each standard duration D, every ride whose stream length >= D is considered.
/// Within each qualifying ride a sliding window of size D scans the stream and the
/// highest average over any consecutive D seconds is recorded.  The best value
/// across all qualifying rides becomes the curve point for that duration.
///
/// Rides shorter than D are excluded entirely, so durations with no qualifying
/// rides return 0 — this keeps the curve honest (no extrapolation from shorter
/// efforts).
///
/// Null samples (power meter dropouts, coasting) are treated as 0 W, which is
/// the convention used by WKO, TrainingPeaks, and most other MMP implementations.
pub fn calculate_power_curve(
    rides: &[Ride],
    streams: &HashMap<i64, Vec<Option<i32>>>,
    weight_kg: f32,
) -> Vec<PowerCurvePoint> {
    DURATIONS
        .iter()
        .map(|&duration| {
            let best_watts = best_power_for_duration(rides, streams, duration);
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

/// Returns the highest sliding-window average across all rides whose stream is
/// at least `duration_seconds` long.  Returns 0.0 if no ride qualifies.
fn best_power_for_duration(
    rides: &[Ride],
    streams: &HashMap<i64, Vec<Option<i32>>>,
    duration_seconds: u32,
) -> f32 {
    let dur = duration_seconds as usize;
    rides
        .iter()
        .filter_map(|ride| {
            let stream = streams.get(&ride.id)?;
            best_window_average(stream, dur)
        })
        .fold(0.0_f32, f32::max)
}

/// Finds the highest average power over any `window`-length consecutive slice
/// of `watts` using prefix sums (O(n)).
///
/// Returns `None` if the stream is shorter than `window`.
/// Null samples are treated as 0 W.
fn best_window_average(watts: &[Option<i32>], window: usize) -> Option<f32> {
    if watts.len() < window {
        return None;
    }

    // Build prefix sums (treating None as 0) so any window sum is O(1).
    let mut prefix: Vec<i64> = vec![0i64; watts.len() + 1];
    for (i, &w) in watts.iter().enumerate() {
        prefix[i + 1] = prefix[i] + w.unwrap_or(0) as i64;
    }

    let best_sum = (0..=(watts.len() - window))
        .map(|i| prefix[i + window] - prefix[i])
        .max()?;

    let avg = best_sum as f32 / window as f32;
    Some((avg * 10.0).round() / 10.0)
}
