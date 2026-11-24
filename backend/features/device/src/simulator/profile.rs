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
        unit_id: Some(UNIT_ID.to_owned()),
    }
}

pub const UNIT_ID: &str = "simulator";
pub const RANGE_COUNT: usize = 12;

/// Zero error in volts on the four most sensitive ranges (2, 5, 10, 20 mV/div), protocol A.3.
const BASE_ZERO_ERROR_V: [f64; 4] = [0.66e-3, 1.04e-3, 0.83e-3, 0.83e-3];
/// Small fixed difference between the channels, volts, added on those ranges.
const CHANNEL_SKEW_V: [f64; 4] = [0.0, 0.05e-3, -0.05e-3, 0.1e-3];
/// Noise standard deviation per range in codes (2 mV/div first), protocol A.3.
const NOISE_SD_CODES: [f64; RANGE_COUNT] = [24.0, 8.6, 4.1, 2.1, 0.9, 0.5, 0.5, 0.05, 0.0, 0.4, 0.0, 0.0];

pub fn default_zero_error_volts() -> [[f64; RANGE_COUNT]; 4] {
    std::array::from_fn(|channel| {
        std::array::from_fn(|range| {
            BASE_ZERO_ERROR_V
                .get(range)
                .map_or(0.0, |base| base + CHANNEL_SKEW_V[channel])
        })
    })
}

/// Noise a shorted input shows on `range`, in volts rms at the input.
pub fn noise_sd_volts(range: usize, volts_per_div: f64, counts_per_div: f64) -> f64 {
    NOISE_SD_CODES.get(range).copied().unwrap_or(0.0) * volts_per_div / counts_per_div
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
        assert_eq!(identity.unit_id.as_deref(), Some("simulator"));
    }

    #[test]
    fn given_the_default_zero_errors_when_read_then_four_sensitive_ranges_have_them_and_the_rest_none() {
        let errors = default_zero_error_volts();

        assert!((errors[0][0] - 0.66e-3).abs() < 1e-12);
        assert!((errors[0][1] - 1.04e-3).abs() < 1e-12);
        assert!(errors[0][4..].iter().all(|v| *v == 0.0));
        assert!(errors.iter().all(|c| c[11] == 0.0));
        assert_ne!(errors[0][0], errors[3][0]);
    }

    #[test]
    fn given_the_two_millivolt_range_when_noise_is_asked_then_it_is_24_codes_in_volts() {
        let volts = noise_sd_volts(0, 0.002, 24.15);

        assert!((volts - 24.0 * 0.002 / 24.15).abs() < 1e-12);
        assert_eq!(noise_sd_volts(11, 10.0, 24.15), 0.0);
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
