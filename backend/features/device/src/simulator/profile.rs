//! What the simulated 6254BC reports about itself: identity and the ADC model; the capability tables are shared (`crate::profile`).

use crate::{CalibrationStatus, DeviceIdentity};

pub use crate::profile::capabilities;

const ZERO_VOLTS_CODE: f64 = 128.0;

pub fn identity() -> DeviceIdentity {
    DeviceIdentity {
        model: "6254BC (simulated)".to_owned(),
        firmware: Some("sim-1.0".to_owned()),
        fpga_version: Some("sim-0x0001".to_owned()),
        calibration: CalibrationStatus::Valid,
        simulated: true,
    }
}

/// ADC channel mode of the hardware: 1, 2 or 4 (three enabled channels use the four channel mode).
pub fn adc_channels(enabled: u8) -> u8 {
    match enabled {
        0 | 1 => 1,
        2 => 2,
        _ => 4,
    }
}

/// Measured on a 6254BC by the driver for each ADC channel mode.
pub fn counts_per_div(adc_channels: u8) -> f64 {
    match adc_channels {
        1 => 21.1,
        2 => 21.6,
        _ => 24.15,
    }
}

pub fn zero_code(position_divs: f64, counts_per_div: f64) -> f64 {
    ZERO_VOLTS_CODE + position_divs * counts_per_div
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_the_simulator_when_identity_is_read_then_it_is_marked_simulated_with_firmware_and_calibration() {
        let identity = identity();

        assert!(identity.model.contains("simulated"));
        assert!(identity.simulated);
        assert!(identity.firmware.is_some() && identity.fpga_version.is_some());
        assert_eq!(identity.calibration, CalibrationStatus::Valid);
    }

    #[test]
    fn given_the_enabled_channel_count_when_the_adc_mode_is_chosen_then_three_uses_four() {
        let modes: Vec<u8> = (1..=4).map(adc_channels).collect();

        assert_eq!(modes, vec![1, 2, 4, 4]);
    }

    #[test]
    fn given_each_adc_mode_when_counts_per_div_is_asked_then_it_matches_the_driver() {
        assert_eq!(counts_per_div(4), 24.15);
        assert_eq!(counts_per_div(2), 21.6);
        assert_eq!(counts_per_div(1), 21.1);
    }
}
