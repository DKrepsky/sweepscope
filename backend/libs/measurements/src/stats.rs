//! Mean and root mean square over a stretch of samples.

pub(crate) fn mean_and_rms(volts: &[f64]) -> (f64, f64) {
    let count = volts.len() as f64;
    let mean = volts.iter().sum::<f64>() / count;
    let rms = (volts.iter().map(|v| v * v).sum::<f64>() / count).sqrt();
    (mean, rms)
}

/// Samples whose index lies inside `[first, last]` (fractional positions); never empty for two distinct crossings.
pub(crate) fn between(volts: &[f64], first: f64, last: f64) -> &[f64] {
    let start = first.ceil() as usize;
    let end = (last.floor() as usize).min(volts.len() - 1);
    if start > end {
        return volts;
    }
    &volts[start..=end]
}
