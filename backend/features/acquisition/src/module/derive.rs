//! Computes the math trace of a captured frame (ADR 0019). Inert until its own package fills it in.
use std::sync::Arc;

use sweepscope_kernel::contracts::{DerivedTrace, Frame};

use crate::Instrument;

pub fn derive_trace(_instrument: &Instrument, _frame: &Frame) -> Option<Arc<DerivedTrace>> {
    None
}
