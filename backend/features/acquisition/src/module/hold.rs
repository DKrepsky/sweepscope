//! The instrument lent to another module (ADR 0021 item 2): what to restore on release and the probe being captured.
use std::sync::Arc;

use sweepscope_kernel::contracts::{CaptureOutcome, Frame, ProbeCaptured};

pub struct Hold {
    /// Whether capturing resumes on release; an armed Single is not re-armed.
    pub was_running: bool,
    /// The device captures on behalf of probes.
    pub capturing: bool,
    pub probe: Option<ProbeRun>,
}

impl Hold {
    pub fn new(was_running: bool) -> Self {
        Self {
            was_running,
            capturing: false,
            probe: None,
        }
    }
}

/// Drops `discard` frames, keeps the next `keep`.
pub struct ProbeRun {
    id: u64,
    pub setup_revision: u64,
    discard_left: u8,
    keep: usize,
    frames: Vec<Arc<Frame>>,
}

impl ProbeRun {
    pub fn new(id: u64, setup_revision: u64, discard: u8, keep: u8) -> Self {
        Self {
            id,
            setup_revision,
            discard_left: discard,
            keep: usize::from(keep),
            frames: Vec::new(),
        }
    }

    /// The answer once the probe is complete or failed; a timeout fails it with no frames.
    pub fn observe(&mut self, outcome: CaptureOutcome) -> Option<ProbeCaptured> {
        let CaptureOutcome::Frame(frame) = outcome else {
            return Some(self.fail());
        };
        if self.discard_left > 0 {
            self.discard_left -= 1;
            return None;
        }
        self.frames.push(frame);
        if self.frames.len() < self.keep {
            return None;
        }
        let frames = std::mem::take(&mut self.frames);
        Some(self.answer(frames))
    }

    pub fn fail(&self) -> ProbeCaptured {
        self.answer(Vec::new())
    }

    fn answer(&self, frames: Vec<Arc<Frame>>) -> ProbeCaptured {
        ProbeCaptured { id: self.id, frames }
    }
}
