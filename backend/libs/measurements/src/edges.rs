//! Rise and fall time between the 10 % and 90 % levels of the base-to-top span.

use crate::crossings::Crossing;
use crate::Invalid;

const MIN_SAMPLES_ON_EDGE: usize = 3;

pub(crate) struct EdgeTimes {
    pub rise: Result<f64, Invalid>,
    pub fall: Result<f64, Invalid>,
}

/// Durations are in seconds.
pub(crate) fn measure(volts: &[f64], crossings: &[Crossing], mid: f64, sample_interval_s: f64) -> EdgeTimes {
    let (Some(base), Some(top)) = (median(volts, |v| v < mid), median(volts, |v| v > mid)) else {
        return EdgeTimes {
            rise: Err(Invalid::NoCompleteEdge),
            fall: Err(Invalid::NoCompleteEdge),
        };
    };
    let flipped: Vec<f64> = volts.iter().map(|v| -v).collect();
    EdgeTimes {
        rise: mean_duration(volts, crossings, true, (base, top), sample_interval_s),
        fall: mean_duration(&flipped, crossings, false, (-top, -base), sample_interval_s),
    }
}

fn mean_duration(
    volts: &[f64],
    crossings: &[Crossing],
    rising: bool,
    (base, top): (f64, f64),
    sample_interval_s: f64,
) -> Result<f64, Invalid> {
    let span = top - base;
    let (low, high) = (base + 0.1 * span, base + 0.9 * span);
    let edges: Vec<Option<(f64, usize)>> = crossings
        .iter()
        .filter(|c| c.rising == rising)
        .map(|c| edge(volts, c.index, low, high))
        .collect();
    let complete: Vec<(f64, usize)> = edges.into_iter().flatten().collect();
    if complete.is_empty() {
        return Err(Invalid::NoCompleteEdge);
    }
    let sharp_enough: Vec<f64> = complete
        .iter()
        .filter(|(_, samples)| *samples >= MIN_SAMPLES_ON_EDGE)
        .map(|(duration, _)| *duration)
        .collect();
    if sharp_enough.is_empty() {
        return Err(Invalid::EdgeTooFast);
    }
    Ok(sharp_enough.iter().sum::<f64>() / sharp_enough.len() as f64 * sample_interval_s)
}

/// Rising edge through the sample pair at `at`: duration in samples and the samples strictly inside it.
fn edge(volts: &[f64], at: usize, low: f64, high: f64) -> Option<(f64, usize)> {
    let mut start = at;
    while start > 0 && volts[start] > low {
        start -= 1;
    }
    let mut end = at + 1;
    while end + 1 < volts.len() && volts[end] < high {
        end += 1;
    }
    if volts[start] > low || volts[end] < high {
        return None;
    }
    let t_low = interpolate(volts, start, low);
    let t_high = interpolate(volts, end - 1, high);
    let first_inside = t_low.floor() as usize + 1;
    let last_inside = t_high.ceil() as usize;
    let inside = last_inside.saturating_sub(first_inside);
    Some((t_high - t_low, inside))
}

fn interpolate(volts: &[f64], index: usize, level: f64) -> f64 {
    let (a, b) = (volts[index], volts[index + 1]);
    if (b - a).abs() > f64::EPSILON {
        index as f64 + ((level - a) / (b - a)).clamp(0.0, 1.0)
    } else {
        index as f64
    }
}

fn median(volts: &[f64], keep: impl Fn(f64) -> bool) -> Option<f64> {
    let mut kept: Vec<f64> = volts.iter().copied().filter(|&v| keep(v)).collect();
    kept.sort_by(f64::total_cmp);
    kept.get(kept.len() / 2).copied()
}
