//! Computes the math trace of a captured frame (ADR 0019).
use std::sync::Arc;

use sweepscope_kernel::contracts::{DerivedTrace, Frame};

use crate::Instrument;

pub fn derive_trace(instrument: &Instrument, frame: &Frame) -> Option<Arc<DerivedTrace>> {
    instrument.derive_math(frame).map(Arc::new)
}
