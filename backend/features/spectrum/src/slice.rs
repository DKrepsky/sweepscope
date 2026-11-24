//! The published slice: revisioned, changes only with a different state, announced at most four times a second.
use std::time::{Duration, Instant};

use crate::messages::SpectrumState;

const MIN_PUBLISH_INTERVAL: Duration = Duration::from_millis(250);

pub struct Slice {
    state: SpectrumState,
    published_at: Option<Instant>,
    unannounced: bool,
}

impl Slice {
    /// `initial` is taken as revision 0.
    pub fn new(initial: SpectrumState) -> Self {
        Self {
            state: SpectrumState { revision: 0, ..initial },
            published_at: None,
            unannounced: false,
        }
    }

    pub fn state(&self) -> SpectrumState {
        self.state.clone()
    }

    /// Returns the state to announce now, if it changed and the rate limit allows it.
    pub fn update(&mut self, next: SpectrumState, now: Instant) -> Option<SpectrumState> {
        let next = SpectrumState {
            revision: self.state.revision,
            ..next
        };
        if next != self.state {
            self.state = SpectrumState {
                revision: next.revision + 1,
                ..next
            };
            self.unannounced = true;
        }
        self.announce_if_due(now)
    }

    /// A change held back by the rate limit becomes due once the interval has passed.
    pub fn announce_if_due(&mut self, now: Instant) -> Option<SpectrumState> {
        let waited_enough = self
            .published_at
            .is_none_or(|at| now.saturating_duration_since(at) >= MIN_PUBLISH_INTERVAL);
        if !(self.unannounced && waited_enough) {
            return None;
        }
        self.unannounced = false;
        self.published_at = Some(now);
        Some(self.state.clone())
    }
}

#[cfg(test)]
mod tests {
    use sweepscope_kernel::contracts::{Availability, FormattedValue};

    use super::*;
    use crate::messages::{SpectrumScale, SpectrumWindow};

    fn state(top_label: &str) -> SpectrumState {
        let value = FormattedValue {
            value: 0.0,
            text: "-".into(),
            unit: String::new(),
        };
        SpectrumState {
            revision: 0,
            active: false,
            window: SpectrumWindow::Hann,
            scale: SpectrumScale::Dbv,
            bin_spacing: value.clone(),
            span: value.clone(),
            sample_rate: value.clone(),
            record_points: value,
            top_label: top_label.into(),
            ticks_x: Vec::new(),
            ticks_y: Vec::new(),
            availability: Availability {
                enabled: true,
                reason: None,
            },
        }
    }

    #[test]
    fn given_a_first_change_when_updated_then_it_is_announced_at_once_with_revision_one() {
        let mut slice = Slice::new(state("a"));

        let announced = slice.update(state("b"), Instant::now()).unwrap();

        assert_eq!(announced.revision, 1);
    }

    #[test]
    fn given_the_same_state_when_updated_then_nothing_is_announced_and_the_revision_stays() {
        let mut slice = Slice::new(state("a"));

        let announced = slice.update(state("a"), Instant::now());

        assert_eq!(announced, None);
        assert_eq!(slice.state().revision, 0);
    }

    #[test]
    fn given_a_change_inside_the_interval_when_it_elapses_then_the_change_is_announced_once() {
        let mut slice = Slice::new(state("a"));
        let start = Instant::now();
        slice.update(state("b"), start);

        let early = slice.update(state("c"), start + Duration::from_millis(100));
        let late = slice.announce_if_due(start + Duration::from_millis(250));
        let again = slice.announce_if_due(start + Duration::from_millis(600));

        assert_eq!(early, None);
        assert_eq!(late.map(|s| s.revision), Some(2));
        assert_eq!(again, None);
    }

    #[test]
    fn given_sixty_changes_in_one_second_when_updated_then_at_most_four_are_announced() {
        let mut slice = Slice::new(state("a"));
        let start = Instant::now();

        let announced = (0..60)
            .filter_map(|i| slice.update(state(&format!("{i}")), start + Duration::from_millis(i * 1000 / 60)))
            .count();

        assert!((1..=4).contains(&announced), "{announced}");
    }
}
