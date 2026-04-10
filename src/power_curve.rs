use std::collections::HashMap;
use rayon::prelude::*;
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
///
/// Rides are processed in parallel via rayon. Each ride fetches its stream once
/// and computes all durations, then results are aggregated sequentially.
pub fn calculate_power_curve(
    rides: &[Ride],
    streams: &HashMap<i64, Vec<Option<i32>>>,
    weight_kg: f32,
) -> Vec<PowerCurvePoint> {
    // Each ride produces a [f32; N_DURATIONS] of its best watts per duration.
    // Rides with no stream entry are skipped.
    let per_ride: Vec<[f32; DURATIONS.len()]> = rides
        .par_iter()
        .filter_map(|ride| {
            let stream = streams.get(&ride.id)?;
            let prefix = build_prefix(stream);
            let mut best = [0.0_f32; DURATIONS.len()];
            for (i, &duration) in DURATIONS.iter().enumerate() {
                if let Some(avg) = best_window_average(&prefix, stream.len(), duration as usize) {
                    best[i] = avg;
                }
            }
            Some(best)
        })
        .collect();

    // Aggregate: for each duration take the max across all rides.
    let mut best_per_duration = [0.0_f32; DURATIONS.len()];
    for ride_bests in &per_ride {
        for (i, &w) in ride_bests.iter().enumerate() {
            best_per_duration[i] = best_per_duration[i].max(w);
        }
    }

    DURATIONS
        .iter()
        .enumerate()
        .map(|(i, &duration)| {
            let best_watts = best_per_duration[i];
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

/// Builds a prefix-sum array from a nullable power stream (treating None as 0 W).
/// The returned vec has length `stream.len() + 1`, with `prefix[0] == 0`.
fn build_prefix(watts: &[Option<i32>]) -> Vec<i64> {
    let mut prefix = vec![0i64; watts.len() + 1];
    for (i, &w) in watts.iter().enumerate() {
        prefix[i + 1] = prefix[i] + w.unwrap_or(0) as i64;
    }
    prefix
}

/// Finds the highest average power over any `window`-length consecutive slice
/// using a pre-built prefix-sum array (O(n) over the stream, O(1) per window).
///
/// `stream_len` must equal `prefix.len() - 1`.
/// Returns `None` if the stream is shorter than `window`.
fn best_window_average(prefix: &[i64], stream_len: usize, window: usize) -> Option<f32> {
    if stream_len < window {
        return None;
    }

    let best_sum = (0..=(stream_len - window))
        .map(|i| prefix[i + window] - prefix[i])
        .max()?;

    let avg = best_sum as f32 / window as f32;
    Some((avg * 10.0).round() / 10.0)
}
