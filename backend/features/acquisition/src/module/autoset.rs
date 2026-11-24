//! Drives an auto-setup run through the device (ADR 0020 item 3). Probes are applied and captured here; their frames
//! are never published, so the screen keeps the last frame until the run ends.
use std::time::Instant;

use sweepscope_kernel::contracts::{CaptureCompleted, CaptureOutcome};

use super::effects::{Call, Effects};
use crate::domain::autoset_run::{AutosetRun, Finish, Next, Plan};
use crate::Instrument;

/// Sends the plan as the next probe. `false` when no setup can be built, which ends the run as failed.
pub fn send_probe(
    instrument: &Instrument,
    run: &mut AutosetRun,
    setup_revision: u64,
    plan: &Plan,
    fx: &mut Effects,
) -> bool {
    let Some(setup) = instrument.autoset_probe_setup(plan) else {
        return false;
    };
    run.wait_for(setup_revision);
    fx.calls.push(Call::Apply(setup_revision, setup));
    true
}

/// A capture during the run: the answer to the awaited probe moves the machine on; the budget is judged first, and
/// anything else (older revisions, timeouts) is dropped while the run keeps waiting.
pub fn consume(run: &mut AutosetRun, completed: CaptureCompleted, now: Instant) -> Option<Next> {
    if let Some(over) = expire(run, now) {
        return Some(over);
    }
    if run.waiting() != Some(completed.setup_revision) {
        return None;
    }
    match completed.outcome {
        CaptureOutcome::Frame(frame) => run.observe(&frame),
        CaptureOutcome::TimedOut => None,
    }
}

pub fn expire(run: &AutosetRun, now: Instant) -> Option<Next> {
    run.expired(now).then_some(Next::Finish(Finish::Failed))
}
