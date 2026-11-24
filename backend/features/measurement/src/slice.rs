//! The published slice: revisioned, changes only with shown text, announced at most four times a second.
use std::time::{Duration, Instant};

use crate::messages::{ChannelMeasurements, MeasurementsState};
use crate::texts;

const MIN_PUBLISH_INTERVAL: Duration = Duration::from_millis(250);

pub struct Slice {
    state: MeasurementsState,
    published_at: Option<Instant>,
    unannounced: bool,
}

impl Slice {
    pub fn new() -> Self {
        Self {
            state: MeasurementsState {
                revision: 0,
                channels: Vec::new(),
                accuracy_note: texts::ACCURACY_NOTE.to_owned(),
                warnings: Vec::new(),
            },
            published_at: None,
            unannounced: false,
        }
    }

    pub fn state(&self) -> MeasurementsState {
        self.state.clone()
    }

    /// Returns the state to announce now, if the text changed and the rate limit allows it.
    pub fn update(&mut self, channels: Vec<ChannelMeasurements>, now: Instant) -> Option<MeasurementsState> {
        if shown_text(&channels) != shown_text(&self.state.channels) {
            self.state.revision += 1;
            self.state.channels = channels;
            self.unannounced = true;
        }
        self.announce_if_due(now)
    }

    /// A change held back by the rate limit becomes due once the interval has passed.
    pub fn announce_if_due(&mut self, now: Instant) -> Option<MeasurementsState> {
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

type Shown<'a> = Vec<(u8, bool, Option<&'a str>, Vec<(&'a str, &'a str, &'a str)>)>;

/// What the user can read: the numbers' text and unit, the invalid reason text, the flags.
fn shown_text(channels: &[ChannelMeasurements]) -> Shown<'_> {
    channels
        .iter()
        .map(|channel| {
            let cells = channel
                .measurements
                .iter()
                .map(|m| match (&m.value, &m.invalid) {
                    (Some(value), _) => (m.label.as_str(), value.text.as_str(), value.unit.as_str()),
                    (None, Some(invalid)) => (m.label.as_str(), invalid.text.as_str(), ""),
                    (None, None) => (m.label.as_str(), "", ""),
                })
                .collect();
            (channel.channel, channel.clipped, channel.unverified.as_deref(), cells)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use sweepscope_kernel::contracts::FormattedValue;

    use super::*;
    use crate::messages::{Measurement, MeasurementId};

    fn channel(text: &str, value: f64) -> Vec<ChannelMeasurements> {
        vec![ChannelMeasurements {
            channel: 1,
            clipped: false,
            unverified: None,
            measurements: vec![Measurement {
                id: MeasurementId::Vpp,
                label: "Vpp".into(),
                value: Some(FormattedValue {
                    value,
                    text: text.into(),
                    unit: "V".into(),
                }),
                invalid: None,
            }],
        }]
    }

    #[test]
    fn given_a_first_change_when_updated_then_it_is_announced_at_once_with_revision_one() {
        let mut slice = Slice::new();

        let announced = slice.update(channel("1.91", 1.91), Instant::now()).unwrap();

        assert_eq!(announced.revision, 1);
    }

    #[test]
    fn given_the_same_text_with_a_different_number_when_updated_then_nothing_is_announced() {
        let mut slice = Slice::new();
        let start = Instant::now();
        slice.update(channel("1.91", 1.911), start);

        let announced = slice.update(channel("1.91", 1.912), start + Duration::from_secs(1));

        assert_eq!(announced, None);
        assert_eq!(slice.state().revision, 1);
    }

    #[test]
    fn given_a_change_inside_the_interval_when_it_elapses_then_the_change_is_announced_once() {
        let mut slice = Slice::new();
        let start = Instant::now();
        slice.update(channel("1.91", 1.91), start);

        let early = slice.update(channel("1.92", 1.92), start + Duration::from_millis(100));
        let late = slice.announce_if_due(start + Duration::from_millis(250));
        let again = slice.announce_if_due(start + Duration::from_millis(600));

        assert_eq!(early, None);
        assert_eq!(late.map(|s| s.revision), Some(2));
        assert_eq!(again, None);
    }

    #[test]
    fn given_sixty_changing_frames_in_one_second_when_updated_then_at_most_four_are_announced() {
        let mut slice = Slice::new();
        let start = Instant::now();

        let announced = (0..60)
            .filter_map(|i| {
                let now = start + Duration::from_millis(i * 1000 / 60);
                slice.update(channel(&format!("{i}"), i as f64), now)
            })
            .count();

        assert!((1..=4).contains(&announced), "{announced}");
    }
}
