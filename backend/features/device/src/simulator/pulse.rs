//! Pulse-width trigger search on a continuous signal, as plain functions.

use super::edge::{find_crossing, Direction};
use crate::PulseCondition;

/// Share of the set width inside which `Equal` holds (the hardware accepts +-5 %, driver docs).
const EQUAL_TOLERANCE: f64 = 0.05;

/// Whether a measured pulse width satisfies the condition against the set width.
pub fn width_matches(measured_s: f64, set_s: f64, condition: PulseCondition) -> bool {
    let equal = (measured_s - set_s).abs() <= EQUAL_TOLERANCE * set_s;
    match condition {
        PulseCondition::Equal => equal,
        PulseCondition::NotEqual => !equal,
        PulseCondition::Greater => measured_s > set_s,
        PulseCondition::Less => measured_s < set_s,
    }
}

/// What a pulse trigger looks for: a leading edge in `leading` direction, a trailing edge back, and a width test.
pub struct PulseSearch<'a> {
    pub level: f64,
    pub leading: Direction,
    pub accepts_width: &'a dyn Fn(f64) -> bool,
    pub step: f64,
}

/// Time of the leading edge of the first pulse that starts in `[from, to]` and has an accepted width.
///
/// A pulse whose trailing edge is not found within `trailing_limit` seconds after its leading edge ends the
/// search: nothing wider can be measured.
pub fn find_pulse(
    value_at: impl Fn(f64) -> f64,
    search: &PulseSearch,
    from: f64,
    to: f64,
    trailing_limit: f64,
) -> Option<f64> {
    let trailing = match search.leading {
        Direction::Rising => Direction::Falling,
        Direction::Falling => Direction::Rising,
    };
    let mut start = from;
    while start < to {
        let lead = find_crossing(&value_at, search.level, search.leading, start, to, search.step)?;
        let end = find_crossing(
            &value_at,
            search.level,
            trailing,
            lead,
            lead + trailing_limit,
            search.step,
        )?;
        if (search.accepts_width)(end - lead) {
            return Some(lead);
        }
        start = end;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulator::signal::{clean, SignalKind, SignalSpec};

    const SET: f64 = 500e-6;

    #[test]
    fn given_a_greater_condition_when_the_width_is_above_the_set_width_then_it_matches() {
        assert!(width_matches(600e-6, SET, PulseCondition::Greater));
        assert!(!width_matches(400e-6, SET, PulseCondition::Greater));
        assert!(!width_matches(SET, SET, PulseCondition::Greater));
    }

    #[test]
    fn given_a_less_condition_when_the_width_is_below_the_set_width_then_it_matches() {
        assert!(width_matches(400e-6, SET, PulseCondition::Less));
        assert!(!width_matches(600e-6, SET, PulseCondition::Less));
    }

    #[test]
    fn given_an_equal_condition_when_the_width_is_within_5_percent_then_it_matches() {
        assert!(width_matches(476e-6, SET, PulseCondition::Equal));
        assert!(width_matches(524e-6, SET, PulseCondition::Equal));
        assert!(!width_matches(450e-6, SET, PulseCondition::Equal));
        assert!(!width_matches(550e-6, SET, PulseCondition::Equal));
    }

    #[test]
    fn given_a_not_equal_condition_when_the_width_is_outside_5_percent_then_it_matches() {
        assert!(width_matches(450e-6, SET, PulseCondition::NotEqual));
        assert!(!width_matches(SET, SET, PulseCondition::NotEqual));
    }

    fn pulses(width_s: f64) -> impl Fn(f64) -> f64 {
        let spec = SignalSpec::new(SignalKind::PulseTrain { width_s });
        move |t| clean(&spec, t)
    }

    fn search(accepts: &dyn Fn(f64) -> bool, leading: Direction) -> Option<f64> {
        let search = PulseSearch {
            level: 0.5,
            leading,
            accepts_width: accepts,
            step: 5e-6,
        };
        find_pulse(pulses(300e-6), &search, 0.0, 2e-3, 1e-3)
    }

    #[test]
    fn given_a_pulse_train_when_any_width_is_accepted_then_a_leading_edge_is_found() {
        let at = search(&|_| true, Direction::Rising).unwrap();

        assert!(at.abs() < 1e-6 || (at - 1e-3).abs() < 1e-6, "{at}");
    }

    #[test]
    fn given_a_width_filter_when_the_pulses_are_too_narrow_then_none_is_found() {
        assert_eq!(search(&|width| width > 400e-6, Direction::Rising), None);
    }

    #[test]
    fn given_the_measured_width_when_the_filter_is_called_then_it_is_the_edge_to_edge_time() {
        let at = search(&|width| (width - 300e-6).abs() < 1e-6, Direction::Rising);

        assert!(at.is_some());
    }

    #[test]
    fn given_a_falling_leading_edge_on_a_positive_train_when_searched_then_the_gap_is_measured() {
        let gap = 1e-3 - 300e-6;

        let at = search(&|width| (width - gap).abs() < 1e-6, Direction::Falling);

        assert!(at.is_some());
    }
}
