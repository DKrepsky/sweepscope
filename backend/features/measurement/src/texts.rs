use measurements::Invalid as LibInvalid;

use crate::messages::{Invalid, InvalidReason, MeasurementId};

pub fn label(id: MeasurementId) -> &'static str {
    match id {
        MeasurementId::Frequency => "Frequency",
        MeasurementId::Period => "Period",
        MeasurementId::Vpp => "Vpp",
        MeasurementId::Vmax => "Vmax",
        MeasurementId::Vmin => "Vmin",
        MeasurementId::Vmean => "Vmean",
        MeasurementId::Vrms => "Vrms",
        MeasurementId::PositiveWidth => "+Width",
        MeasurementId::NegativeWidth => "-Width",
        MeasurementId::DutyCycle => "Duty",
        MeasurementId::RiseTime => "Rise time",
        MeasurementId::FallTime => "Fall time",
    }
}

pub fn invalid(reason: InvalidReason) -> Invalid {
    let text = match reason {
        InvalidReason::NoSignal => "Signal is flat or too small",
        InvalidReason::FewerThanOnePeriod => "Fewer than one period in the record",
        InvalidReason::NoCompleteEdge => "No complete edge in the record",
        InvalidReason::EdgeTooFast => "Edge too fast for the sample interval (needs 3 samples across the edge)",
        InvalidReason::NoCompletePulse => "No complete pulse in the record",
        InvalidReason::Clipped => "Signal is clipped at the converter limits",
    };
    Invalid {
        reason,
        text: text.to_owned(),
    }
}

pub fn reason_of(lib: LibInvalid) -> InvalidReason {
    match lib {
        LibInvalid::NoSignal => InvalidReason::NoSignal,
        LibInvalid::FewerThanOnePeriod => InvalidReason::FewerThanOnePeriod,
        LibInvalid::NoCompleteEdge => InvalidReason::NoCompleteEdge,
        LibInvalid::EdgeTooFast => InvalidReason::EdgeTooFast,
        LibInvalid::NoCompletePulse => InvalidReason::NoCompletePulse,
    }
}
