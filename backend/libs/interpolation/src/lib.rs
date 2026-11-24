//! Band-limited (sin(x)/x) upsampling of a window of a sample record (ADR 0015).
#![forbid(unsafe_code)]

use std::f64::consts::PI;

/// Taps on each side of the interpolated point.
const LOBES: isize = 8;

/// Returns `(len - 1) * factor + 1` samples; `output[k * factor] == record[start + k]`.
/// Needs `factor >= 1` and `len >= 2`; reads the record outside the window to avoid edge ringing.
pub fn upsample_window(record: &[f32], start: usize, len: usize, factor: usize) -> Vec<f32> {
    assert!(factor >= 1 && len >= 2, "factor >= 1 and len >= 2 required");
    assert!(start + len <= record.len(), "window outside the record");
    let kernels: Vec<Vec<f64>> = (1..factor).map(|phase| kernel(phase as f64 / factor as f64)).collect();
    let mut output = Vec::with_capacity((len - 1) * factor + 1);
    for k in 0..len - 1 {
        let base = start + k;
        output.push(record[base]);
        output.extend(kernels.iter().map(|weights| convolve(record, base, weights)));
    }
    output.push(record[start + len - 1]);
    output
}

/// Normalised Lanczos weights for the taps `base - LOBES + 1 ..= base + LOBES` at fraction `frac` after `base`.
fn kernel(frac: f64) -> Vec<f64> {
    let mut weights: Vec<f64> = (1 - LOBES..=LOBES).map(|tap| lanczos(frac - tap as f64)).collect();
    let sum: f64 = weights.iter().sum();
    weights.iter_mut().for_each(|w| *w /= sum);
    weights
}

fn lanczos(x: f64) -> f64 {
    if x.abs() >= LOBES as f64 {
        0.0
    } else {
        sinc(x) * sinc(x / LOBES as f64)
    }
}

fn sinc(x: f64) -> f64 {
    if x == 0.0 {
        1.0
    } else {
        (PI * x).sin() / (PI * x)
    }
}

fn convolve(record: &[f32], base: usize, weights: &[f64]) -> f32 {
    let last = record.len() as isize - 1;
    weights
        .iter()
        .zip(1 - LOBES..)
        .map(|(w, tap)| w * f64::from(record[(base as isize + tap).clamp(0, last) as usize]))
        .sum::<f64>() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(points: usize, per_cycle: f64) -> Vec<f32> {
        (0..points)
            .map(|i| (2.0 * PI * i as f64 / per_cycle).sin() as f32)
            .collect()
    }

    #[test]
    fn given_a_window_when_upsampled_by_8_then_the_grid_points_equal_the_input_exactly() {
        let record = sine(64, 5.3);

        let out = upsample_window(&record, 10, 20, 8);

        assert_eq!(out.len(), 19 * 8 + 1);
        assert!((0..20).all(|k| out[k * 8] == record[10 + k]));
    }

    #[test]
    fn given_a_sine_at_4_points_per_cycle_when_upsampled_by_16_then_it_is_within_2_percent() {
        let record = sine(128, 4.0);

        let out = upsample_window(&record, 40, 30, 16);

        let worst = out
            .iter()
            .enumerate()
            .map(|(i, v)| (f64::from(*v) - (2.0 * PI * (40.0 + i as f64 / 16.0) / 4.0).sin()).abs())
            .fold(0.0, f64::max);
        assert!(worst < 0.02, "{worst}");
    }

    #[test]
    fn given_a_window_at_the_start_of_the_record_when_upsampled_then_it_stays_in_range_and_finite() {
        let record = sine(20, 6.0);

        let out = upsample_window(&record, 0, 20, 4);

        assert_eq!(out.len(), 19 * 4 + 1);
        assert!(out.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn given_a_window_at_the_end_of_the_record_when_upsampled_then_it_stays_in_range() {
        let record = sine(20, 6.0);

        let out = upsample_window(&record, 12, 8, 4);

        assert_eq!(out.len(), 7 * 4 + 1);
    }

    #[test]
    fn given_a_factor_of_one_when_upsampled_then_the_window_is_returned() {
        let record = sine(30, 7.0);

        assert_eq!(upsample_window(&record, 5, 10, 1), record[5..15].to_vec());
    }

    #[test]
    fn given_a_constant_record_when_upsampled_then_every_point_is_that_constant() {
        let out = upsample_window(&[0.25; 40], 10, 10, 7);

        assert!(out.iter().all(|v| (v - 0.25).abs() < 1e-6));
    }

    #[test]
    fn given_samples_outside_the_window_when_they_change_then_the_edge_points_change() {
        let mut record = sine(60, 4.0);
        let before = upsample_window(&record, 20, 10, 8);
        record[19] += 1.0;

        let after = upsample_window(&record, 20, 10, 8);

        assert_ne!(before[1], after[1]);
    }
}
