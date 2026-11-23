use std::collections::VecDeque;
use std::time::{Duration, Instant};

const WINDOW: Duration = Duration::from_secs(2);

/// Frames per second over the last two seconds, measured on the instants the injected clock gives.
#[derive(Debug, Default)]
pub struct CaptureMeter {
    frames: VecDeque<Instant>,
}

impl CaptureMeter {
    pub fn record(&mut self, now: Instant) {
        self.frames.push_back(now);
        while self.frames.front().is_some_and(|&t| now.duration_since(t) > WINDOW) {
            self.frames.pop_front();
        }
    }

    pub fn clear(&mut self) {
        self.frames.clear();
    }

    /// `None` until two frames lie within the window.
    pub fn rate(&self, now: Instant) -> Option<f64> {
        let recent: Vec<Instant> = self
            .frames
            .iter()
            .copied()
            .filter(|&t| now.saturating_duration_since(t) <= WINDOW)
            .collect();
        let span = recent.last()?.duration_since(*recent.first()?).as_secs_f64();
        (recent.len() >= 2 && span > 0.0).then(|| (recent.len() - 1) as f64 / span)
    }

    /// Percent of the time the scope was not sampling: `1 - rate x record duration`, clamped to 0 to 100.
    pub fn dead_time_percent(&self, now: Instant, record_duration_s: f64) -> Option<f64> {
        let rate = self.rate(now)?;
        Some(((1.0 - rate * record_duration_s) * 100.0).clamp(0.0, 100.0))
    }
}
