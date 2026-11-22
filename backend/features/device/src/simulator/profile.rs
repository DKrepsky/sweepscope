//! What the simulated 6254BC reports about itself: identity and capability tables (PRD 5.1).

use crate::{CalibrationStatus, Capabilities, DeviceIdentity, TimeBaseStep, VerticalRange};

pub const RECORD_LEN: u32 = 4096;
const STREAMING_FROM_INDEX: usize = 25;
const VERIFIED_RANGES: std::ops::RangeInclusive<usize> = 4..=8;
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

pub fn capabilities() -> Capabilities {
    Capabilities {
        channel_count: 4,
        record_len: RECORD_LEN,
        time_bases: time_bases(),
        vertical_ranges: vertical_ranges(),
        adc_interval_ns: vec![1.0, 2.0, 4.0, 4.0],
        samples_per_div: 250.0,
    }
}

/// 1-2-5 sequence from 2 ns to 1000 s per division, built from integers so the values are the nearest doubles.
fn time_bases() -> Vec<TimeBaseStep> {
    let mut steps: Vec<(i32, f64)> = vec![(-9, 2.0), (-9, 5.0)];
    for exponent in -8..=2 {
        steps.extend([1.0, 2.0, 5.0].map(|mantissa| (exponent, mantissa)));
    }
    steps.push((3, 1.0));
    steps
        .into_iter()
        .enumerate()
        .map(|(index, (exponent, mantissa))| TimeBaseStep {
            seconds_per_div: scaled(mantissa, exponent),
            streaming: index >= STREAMING_FROM_INDEX,
            verified: true,
        })
        .collect()
}

/// 2, 5, 10 mV and so on up to 10 V per division.
fn vertical_ranges() -> Vec<VerticalRange> {
    let mut volts = Vec::new();
    for exponent in -3..=0 {
        volts.extend([1.0, 2.0, 5.0].map(|mantissa| scaled(mantissa, exponent)));
    }
    volts.push(10.0);
    volts
        .into_iter()
        .skip(1)
        .enumerate()
        .map(|(index, volts_per_div)| VerticalRange {
            volts_per_div,
            verified: VERIFIED_RANGES.contains(&index),
        })
        .collect()
}

fn scaled(mantissa: f64, exponent: i32) -> f64 {
    if exponent < 0 {
        mantissa / 10f64.powi(-exponent)
    } else {
        mantissa * 10f64.powi(exponent)
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
    fn given_the_capabilities_when_read_then_there_are_36_time_bases_from_2_ns_to_1000_s() {
        let bases = capabilities().time_bases;

        assert_eq!(bases.len(), 36);
        assert_eq!(bases[0].seconds_per_div, 2e-9);
        assert_eq!(bases[35].seconds_per_div, 1000.0);
    }

    #[test]
    fn given_the_time_bases_when_read_then_streaming_starts_at_500_ms_per_div() {
        let bases = capabilities().time_bases;

        assert_eq!(bases[25].seconds_per_div, 0.5);
        assert!(bases[25].streaming);
        assert!(!bases[24].streaming);
        assert_eq!(bases[24].seconds_per_div, 0.2);
    }

    #[test]
    fn given_the_vertical_ranges_when_read_then_there_are_12_and_only_50_mv_to_1_v_are_verified() {
        let ranges = capabilities().vertical_ranges;

        assert_eq!(ranges.len(), 12);
        assert_eq!(ranges[0].volts_per_div, 0.002);
        assert_eq!(ranges[11].volts_per_div, 10.0);
        let verified: Vec<f64> = ranges.iter().filter(|r| r.verified).map(|r| r.volts_per_div).collect();
        assert_eq!(verified, vec![0.05, 0.1, 0.2, 0.5, 1.0]);
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
