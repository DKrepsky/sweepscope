use sweepscope_kernel::contracts::{AcquisitionSetup, FrameCaptured};

use crate::{AcquisitionStatus, InstrumentFacts, InstrumentState};

/// What a state change publishes.
pub enum Output {
    Instrument(Box<InstrumentState>),
    Status(AcquisitionStatus),
    Facts(Box<InstrumentFacts>),
    Frame(FrameCaptured),
}

/// What a state change asks of the device.
pub enum Call {
    Apply(u64, AcquisitionSetup),
    Capturing(bool),
    Force,
}

/// The result of one step on the core, handed out after the state lock is released.
#[derive(Default)]
pub struct Effects {
    pub outputs: Vec<Output>,
    pub calls: Vec<Call>,
}
