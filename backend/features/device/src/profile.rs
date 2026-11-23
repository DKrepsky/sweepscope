//! Capability tables of the 6254BC (PRD 5.1), shared by the simulator and the hardware adapter.

use crate::{Capabilities, TimeBaseStep, VerticalRange};

pub const RECORD_LEN: u32 = 4096;
const STREAMING_FROM_INDEX: usize = 25;
/// PRD 5.1: time bases measured on the instrument.
const VERIFIED_TIME_BASES: [std::ops::RangeInclusive<usize>; 4] = [0..=22, 24..=25, 27..=27, 30..=30];
const VERIFIED_RANGES: std::ops::RangeInclusive<usize> = 4..=8;
/// ADR 0012 item 3: the position byte spans about 4 divisions, but only 1.6 on these ranges (volts per div).
const NARROW_POSITION_RANGES: [f64; 3] = [0.05, 0.5, 5.0];
const WIDE_POSITION_DIVS: f64 = 4.0;
const NARROW_POSITION_DIVS: f64 = 1.6;

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
            verified: VERIFIED_TIME_BASES.iter().any(|verified| verified.contains(&index)),
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
            max_position_divs: max_position_divs(volts_per_div),
        })
        .collect()
}

fn max_position_divs(volts_per_div: f64) -> f64 {
    if NARROW_POSITION_RANGES.contains(&volts_per_div) {
        NARROW_POSITION_DIVS
    } else {
        WIDE_POSITION_DIVS
    }
}

fn scaled(mantissa: f64, exponent: i32) -> f64 {
    if exponent < 0 {
        mantissa / 10f64.powi(-exponent)
    } else {
        mantissa * 10f64.powi(exponent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn given_the_time_bases_when_read_then_only_the_prd_measured_indexes_are_verified() {
        let verified: Vec<usize> = capabilities()
            .time_bases
            .iter()
            .enumerate()
            .filter(|(_, step)| step.verified)
            .map(|(index, _)| index)
            .collect();

        let expected: Vec<usize> = (0..=22).chain([24, 25, 27, 30]).collect();
        assert_eq!(verified, expected);
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
    fn given_the_vertical_ranges_when_read_then_50_mv_500_mv_and_5_v_reach_1_6_divisions_and_the_rest_4() {
        let ranges = capabilities().vertical_ranges;

        let narrow: Vec<f64> = ranges
            .iter()
            .filter(|r| r.max_position_divs == 1.6)
            .map(|r| r.volts_per_div)
            .collect();
        assert_eq!(narrow, vec![0.05, 0.5, 5.0]);
        assert!(ranges
            .iter()
            .filter(|r| !narrow.contains(&r.volts_per_div))
            .all(|r| r.max_position_divs == 4.0));
    }
}
