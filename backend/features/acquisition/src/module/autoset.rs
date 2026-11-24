//! Drives an auto-setup run through the device (ADR 0020). Inert until its own package fills it in.
use super::effects::Effects;
use crate::domain::autoset_run::AutosetRun;

#[allow(
    dead_code,
    reason = "called by the handler of StartAutoset, which the auto-setup package registers"
)]
pub fn start(_run: &mut AutosetRun, _fx: &mut Effects) {}
