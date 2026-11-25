//! Accuracy texts and availability reasons. Numbers derive from protocol A.2 and A.3 measurements on one unit.
use sweepscope_kernel::contracts::Availability;

pub const NO_DEVICE: &str = "No device";
pub const NEEDS_READY_SCOPE: &str = "Zero calibration needs a ready scope";
pub const NO_SERIAL: &str = "This scope reports no serial number";
pub const AUTO_SETUP_RUNNING: &str = "Auto-setup is running";

pub const TYPICAL_HEADING: &str = "Typical zero error:";
pub const ZERO_REMOVED_HEADING: &str =
    "Zero error removed per channel and range, measured with the input shorted (about 12 000 samples averaged).";

pub fn available() -> Availability {
    Availability {
        enabled: true,
        reason: None,
    }
}

pub fn unavailable(reason: &str) -> Availability {
    Availability {
        enabled: false,
        reason: Some(reason.to_owned()),
    }
}

pub fn heading(calibrated: bool) -> &'static str {
    if calibrated {
        ZERO_REMOVED_HEADING
    } else {
        TYPICAL_HEADING
    }
}
