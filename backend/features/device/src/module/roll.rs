use std::sync::Arc;
use std::time::{Duration, Instant};

use sweepscope_kernel::contracts::RollPolledItem;

use crate::Frame;

/// How often the worker reads the instrument in roll: far inside the RAM span (32.8 s with three or four channels).
pub const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Counts samples of one roll session from index 0 and turns an overrun into a gap with an estimated size (ADR 0009, 0017).
pub struct RollCounter {
    next_index: u64,
    last_poll: Instant,
    sample_interval_s: f64,
}

impl RollCounter {
    pub fn new(now: Instant, sample_interval_s: f64) -> Self {
        Self {
            next_index: 0,
            last_poll: now,
            sample_interval_s,
        }
    }

    /// `None` when the poll returned no new sample.
    pub fn samples(&mut self, now: Instant, frame: Frame) -> Option<RollPolledItem> {
        self.last_poll = now;
        let count = frame
            .channels
            .iter()
            .flatten()
            .map(|channel| channel.codes.len())
            .max()?;
        if count == 0 {
            return None;
        }
        let first_index = self.next_index;
        self.next_index += count as u64;
        Some(RollPolledItem::Samples {
            first_index,
            frame: Arc::new(frame),
        })
    }

    pub fn gap(&mut self, now: Instant) -> RollPolledItem {
        let elapsed = now.saturating_duration_since(self.last_poll).as_secs_f64();
        let missed = (elapsed / self.sample_interval_s.max(f64::MIN_POSITIVE)).floor() as u64;
        let after = self.next_index.saturating_sub(1);
        self.next_index += missed;
        self.last_poll = now;
        RollPolledItem::Gap { after, missed }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChannelData, TriggerOutcome};

    fn frame(count: usize) -> Frame {
        let channel = ChannelData {
            codes: vec![128; count],
            volts_per_div: 1.0,
            zero_code: 128.0,
        };
        Frame {
            channels: [Some(channel), None, None, None],
            counts_per_div: 25.0,
            sample_interval_s: 0.002,
            trigger: TriggerOutcome::None,
        }
    }

    fn first_index(item: Option<RollPolledItem>) -> u64 {
        match item {
            Some(RollPolledItem::Samples { first_index, .. }) => first_index,
            other => panic!("expected samples, got {other:?}"),
        }
    }

    #[test]
    fn given_two_polls_when_counted_then_first_indices_are_contiguous_from_zero() {
        let now = Instant::now();
        let mut counter = RollCounter::new(now, 0.002);

        let first = first_index(counter.samples(now, frame(25)));
        let second = first_index(counter.samples(now, frame(30)));

        assert_eq!((first, second), (0, 25));
    }

    #[test]
    fn given_an_empty_poll_when_counted_then_nothing_is_published_and_the_index_stays() {
        let now = Instant::now();
        let mut counter = RollCounter::new(now, 0.002);

        assert!(counter.samples(now, frame(0)).is_none());

        assert_eq!(first_index(counter.samples(now, frame(5))), 0);
    }

    #[test]
    fn given_an_overrun_after_samples_when_gapped_then_after_is_the_last_index_and_the_next_samples_skip_the_estimate()
    {
        let start = Instant::now();
        let mut counter = RollCounter::new(start, 0.002);
        counter.samples(start, frame(25));

        let gap = counter.gap(start + Duration::from_millis(100));
        let next = first_index(counter.samples(start + Duration::from_millis(150), frame(10)));

        assert_eq!(gap, RollPolledItem::Gap { after: 24, missed: 50 });
        assert_eq!(next, 25 + 50);
    }

    #[test]
    fn given_two_overruns_when_gapped_then_the_second_estimate_counts_from_the_first() {
        let start = Instant::now();
        let mut counter = RollCounter::new(start, 0.002);
        counter.gap(start + Duration::from_millis(100));

        let gap = counter.gap(start + Duration::from_millis(120));

        assert_eq!(gap, RollPolledItem::Gap { after: 49, missed: 10 });
    }
}
