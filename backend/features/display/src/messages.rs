use std::sync::Arc;

use sweepscope_kernel::Event;

/// One encoded wire frame (ADR 0011); latest-wins subscribers only see the newest.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayFrameReady {
    pub bytes: Arc<Vec<u8>>,
}
impl Event for DisplayFrameReady {}
