//! State of one auto-setup run (ADR 0020). Inert until its own package fills it in: the status is always idle.
use crate::AutosetStatus;

#[derive(Debug, Default)]
pub struct AutosetRun;

impl AutosetRun {
    pub fn status(&self) -> AutosetStatus {
        AutosetStatus::default()
    }
}
