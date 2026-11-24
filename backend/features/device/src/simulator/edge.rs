//! Edge trigger search on a continuous signal, as plain functions.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Rising,
    Falling,
}

/// First time in `[from, to]` where `value_at` crosses `level` in `direction`, scanning in `step` seconds.
///
/// A rising crossing is a step from below `level` to at or above it (falling: from above to at or below).
/// The returned time is linearly interpolated inside the step.
pub fn find_crossing(
    value_at: impl Fn(f64) -> f64,
    level: f64,
    direction: Direction,
    from: f64,
    to: f64,
    step: f64,
) -> Option<f64> {
    let mut t = from;
    let mut previous = value_at(t);
    while t < to {
        let next_t = (t + step).min(to);
        let next = value_at(next_t);
        if crosses(previous, next, level, direction) {
            return Some(interpolate(t, next_t, previous, next, level));
        }
        t = next_t;
        previous = next;
    }
    None
}

fn crosses(before: f64, after: f64, level: f64, direction: Direction) -> bool {
    match direction {
        Direction::Rising => before < level && after >= level,
        Direction::Falling => before > level && after <= level,
    }
}

fn interpolate(t0: f64, t1: f64, v0: f64, v1: f64, level: f64) -> f64 {
    t0 + (t1 - t0) * (level - v0) / (v1 - v0)
}

/// Sample index where the trigger sits: `percent` of the record, kept inside it.
pub fn trigger_index(horizontal_position_percent: f64, record_len: u32) -> usize {
    let last = record_len.saturating_sub(1) as f64;
    (horizontal_position_percent / 100.0 * f64::from(record_len))
        .round()
        .clamp(0.0, last) as usize
}

/// Time of the first sample so that sample `trigger_index` lands on `crossing`.
pub fn record_start(crossing: f64, trigger_index: usize, sample_interval_s: f64) -> f64 {
    crossing - trigger_index as f64 * sample_interval_s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulator::signal::{clean, SignalKind, SignalSpec};

    fn ramp(t: f64) -> f64 {
        t * 1000.0
    }

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn given_a_rising_ramp_when_crossing_is_searched_then_the_interpolated_time_is_returned() {
        let t = find_crossing(ramp, 2.5, Direction::Rising, 0.0, 0.01, 0.003).unwrap();
        assert!(near(t, 0.0025));
    }

    #[test]
    fn given_a_rising_ramp_when_a_falling_crossing_is_searched_then_none_is_found() {
        assert_eq!(find_crossing(ramp, 2.5, Direction::Falling, 0.0, 0.01, 0.001), None);
    }

    #[test]
    fn given_a_sine_when_the_falling_zero_crossing_is_searched_then_it_is_at_half_a_period() {
        let t = find_crossing(
            |t| clean(&SignalSpec::new(SignalKind::Sine), t),
            0.0,
            Direction::Falling,
            0.0,
            0.002,
            1e-5,
        )
        .unwrap();
        assert!((t - 0.0005).abs() < 1e-7);
    }

    #[test]
    fn given_a_start_after_the_first_crossing_when_searched_then_the_next_crossing_is_returned() {
        let t = find_crossing(
            |t| clean(&SignalSpec::new(SignalKind::Sine), t),
            0.0,
            Direction::Rising,
            0.0001,
            0.002,
            1e-5,
        )
        .unwrap();
        assert!((t - 0.001).abs() < 1e-7);
    }

    #[test]
    fn given_a_level_above_the_signal_when_searched_then_none_is_found() {
        assert_eq!(
            find_crossing(
                |t| clean(&SignalSpec::new(SignalKind::Sine), t),
                5.0,
                Direction::Rising,
                0.0,
                0.01,
                1e-5
            ),
            None
        );
    }

    #[test]
    fn given_a_window_shorter_than_the_step_when_searched_then_the_end_is_still_checked() {
        let t = find_crossing(ramp, 0.5, Direction::Rising, 0.0, 0.001, 0.01).unwrap();
        assert!(near(t, 0.0005));
    }

    #[test]
    fn given_a_signal_already_above_the_level_at_the_start_when_searched_then_no_rising_crossing_is_reported() {
        assert_eq!(find_crossing(|_| 3.0, 2.0, Direction::Rising, 0.0, 0.01, 0.001), None);
    }

    #[test]
    fn given_a_percentage_when_the_trigger_index_is_asked_then_it_is_that_share_of_the_record() {
        assert_eq!(trigger_index(50.0, 4096), 2048);
        assert_eq!(trigger_index(0.0, 4096), 0);
    }

    #[test]
    fn given_100_percent_when_the_trigger_index_is_asked_then_it_stays_inside_the_record() {
        assert_eq!(trigger_index(100.0, 4096), 4095);
        assert_eq!(trigger_index(150.0, 4096), 4095);
    }

    #[test]
    fn given_a_crossing_when_the_record_start_is_asked_then_the_trigger_sample_lands_on_it() {
        let start = record_start(1.0, 2048, 1e-6);
        assert!(near(start + 2048.0 * 1e-6, 1.0));
    }
}
