//! Wire layout of ADR 0011 (WP7 fills this in).

use sweepscope_kernel::contracts::{Frame, FrameContext};

/// ADR 0011 layout; the shell overwrites bytes 16..24 with the send time.
pub fn encode(_frame: &Frame, _context: &FrameContext, _sequence: u32, _produced_us: f64) -> Vec<u8> {
    todo!("WP7")
}

pub fn stamp_sent(_bytes: &mut [u8], _sent_us: f64) {
    todo!("WP7")
}
