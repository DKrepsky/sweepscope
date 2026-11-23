//! Interpolation rules of ADR 0015: which mode an automatic state shows and how far the window is upsampled.
use crate::messages::InterpolationMode;

const DIVISIONS: usize = 10;
const TARGET_SAMPLES_PER_DIV: usize = 250;
const MAX_FACTOR: usize = 512;
const TARGET_INTERVALS: usize = TARGET_SAMPLES_PER_DIV * DIVISIONS;

/// Sinc when the window holds fewer than 250 samples per division, otherwise linear; linear when nothing can be refined.
pub(crate) fn automatic_mode(window_len: usize) -> InterpolationMode {
    let intervals = window_len.saturating_sub(1);
    if (1..TARGET_INTERVALS).contains(&intervals) {
        InterpolationMode::Sinc
    } else {
        InterpolationMode::Linear
    }
}

/// `clamp(ceil(250 / samples per division), 1, 512)` with `samples per division = (len - 1) / 10`, in integers.
pub(crate) fn upsample_factor(window_len: usize) -> usize {
    let intervals = window_len.saturating_sub(1).max(1);
    TARGET_INTERVALS.div_ceil(intervals).clamp(1, MAX_FACTOR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_2501_points_when_the_automatic_mode_is_chosen_then_it_is_linear() {
        assert_eq!(automatic_mode(2501), InterpolationMode::Linear);
    }

    #[test]
    fn given_2500_points_when_the_automatic_mode_is_chosen_then_it_is_sinc() {
        assert_eq!(automatic_mode(2500), InterpolationMode::Sinc);
    }

    #[test]
    fn given_a_window_that_cannot_be_refined_when_the_automatic_mode_is_chosen_then_it_is_linear() {
        assert_eq!(automatic_mode(1), InterpolationMode::Linear);
        assert_eq!(automatic_mode(0), InterpolationMode::Linear);
    }

    #[test]
    fn given_5_samples_per_division_when_the_factor_is_computed_then_it_is_50() {
        assert_eq!(upsample_factor(51), 50);
    }

    #[test]
    fn given_50_samples_per_division_when_the_factor_is_computed_then_it_is_5() {
        assert_eq!(upsample_factor(501), 5);
    }

    #[test]
    fn given_six_points_when_the_factor_is_computed_then_it_is_500() {
        assert_eq!(upsample_factor(6), 500);
    }

    #[test]
    fn given_very_few_points_when_the_factor_is_computed_then_it_is_capped_at_512() {
        assert_eq!(upsample_factor(3), 512);
    }

    #[test]
    fn given_a_full_window_when_the_factor_is_computed_then_it_is_1() {
        assert_eq!(upsample_factor(2501), 1);
        assert_eq!(upsample_factor(5000), 1);
    }
}
