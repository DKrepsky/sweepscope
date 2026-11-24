use measurements::{measure, Invalid};
use std::f64::consts::TAU;

const VOLTS_PER_COUNT: f64 = 0.03;
const RECORD: usize = 4096;

fn square(period: usize, phase: usize, ramp: usize, level: f64) -> Vec<f64> {
    (0..RECORD)
        .map(|i| {
            let at = (i + period - phase % period) % period;
            let half = period / 2;
            let fraction = |since: usize| (since as f64 / ramp.max(1) as f64).min(1.0);
            if at < half {
                level * if ramp == 0 { 1.0 } else { fraction(at + 1) }
            } else {
                level * (1.0 - if ramp == 0 { 1.0 } else { fraction(at - half + 1) })
            }
        })
        .collect()
}

fn sine(samples_per_period: f64, amplitude: f64, offset: f64) -> Vec<f64> {
    (0..RECORD)
        .map(|i| offset + amplitude * (TAU * i as f64 / samples_per_period).sin())
        .collect()
}

fn triangle(period: usize, amplitude: f64) -> Vec<f64> {
    (0..RECORD)
        .map(|i| {
            let phase = (i % period) as f64 / period as f64;
            amplitude * (1.0 - 4.0 * (phase - 0.5).abs()) / 2.0 + amplitude / 2.0
        })
        .collect()
}

fn near(actual: f64, expected: f64, relative: f64) -> bool {
    (actual - expected).abs() <= expected.abs() * relative
}

#[test]
fn given_a_1_khz_square_when_measured_then_frequency_period_duty_and_widths_match() {
    let interval = 800e-9;
    let record = square(1250, 100, 0, 1.9);

    let measured = measure(&record, interval, VOLTS_PER_COUNT);

    assert!(near(measured.frequency.unwrap(), 1000.0, 1e-4));
    assert!(near(measured.period.unwrap(), 1e-3, 1e-4));
    assert!((measured.duty_cycle.unwrap() - 50.0).abs() < 0.5);
    assert!(near(measured.positive_width.unwrap(), 500e-6, 1e-3));
    assert!(near(measured.negative_width.unwrap(), 500e-6, 1e-3));
    assert!(near(measured.vpp, 1.9, 1e-9));
    assert_eq!((measured.vmax, measured.vmin), (1.9, 0.0));
}

#[test]
fn given_a_square_with_a_quarter_duty_when_measured_then_duty_cycle_is_25_percent() {
    let record: Vec<f64> = (0..RECORD).map(|i| if i % 400 < 100 { 1.0 } else { 0.0 }).collect();

    let measured = measure(&record, 1e-6, VOLTS_PER_COUNT);

    assert!((measured.duty_cycle.unwrap() - 25.0).abs() < 0.5);
}

#[test]
fn given_a_sine_with_a_partial_last_period_when_measured_then_mean_uses_whole_periods() {
    let record = sine(RECORD as f64 / 3.28, 1.0, 0.0);

    let measured = measure(&record, 1e-6, VOLTS_PER_COUNT);

    assert!(measured.vmean.unwrap().abs() < 0.01);
    assert!(near(measured.vrms.unwrap(), 1.0 / 2f64.sqrt(), 0.01));
}

#[test]
fn given_sines_with_whole_periods_when_measured_then_frequency_is_within_0_01_percent() {
    for periods in 3..=40 {
        let samples_per_period = RECORD as f64 / periods as f64;
        let record = sine(samples_per_period, 1.0, 0.1);
        let interval = 1e-6;

        let measured = measure(&record, interval, VOLTS_PER_COUNT);

        let expected = 1.0 / (samples_per_period * interval);
        assert!(
            near(measured.frequency.unwrap(), expected, 1e-4),
            "{periods} periods: {:?} vs {expected}",
            measured.frequency
        );
    }
}

#[test]
fn given_a_dc_level_when_measured_then_mean_is_the_level_and_timing_is_no_signal() {
    let measured = measure(&vec![1.5; RECORD], 1e-6, VOLTS_PER_COUNT);

    assert_eq!(measured.vmean, Ok(1.5));
    assert!((measured.vrms.unwrap() - 1.5).abs() < 1e-12);
    assert_eq!(measured.frequency, Err(Invalid::NoSignal));
    assert_eq!(measured.positive_width, Err(Invalid::NoSignal));
    assert_eq!(measured.rise_time, Err(Invalid::NoSignal));
}

#[test]
fn given_a_flat_line_when_measured_then_timing_is_no_signal_and_vpp_is_zero() {
    let measured = measure(&vec![0.0; RECORD], 1e-6, VOLTS_PER_COUNT);

    assert_eq!(measured.vpp, 0.0);
    assert_eq!(measured.period, Err(Invalid::NoSignal));
    assert_eq!(measured.duty_cycle, Err(Invalid::NoSignal));
    assert_eq!(measured.fall_time, Err(Invalid::NoSignal));
}

#[test]
fn given_a_swing_below_ten_counts_when_measured_then_timing_is_no_signal() {
    let record = sine(100.0, 0.1 * VOLTS_PER_COUNT * 4.0, 0.0);

    let measured = measure(&record, 1e-6, VOLTS_PER_COUNT);

    assert_eq!(measured.frequency, Err(Invalid::NoSignal));
    assert!(measured.vpp > 0.0);
}

#[test]
fn given_less_than_one_period_when_measured_then_frequency_is_invalid_and_amplitudes_are_valid() {
    let record = sine(RECORD as f64 * 2.0, 1.0, 0.0);

    let measured = measure(&record, 1e-6, VOLTS_PER_COUNT);

    assert_eq!(measured.frequency, Err(Invalid::FewerThanOnePeriod));
    assert_eq!(measured.period, Err(Invalid::FewerThanOnePeriod));
    assert!(near(measured.vpp, 1.0, 0.01));
    assert!(measured.vmean.is_ok());
}

#[test]
fn given_a_single_rising_edge_when_measured_then_no_pulse_is_complete() {
    let record: Vec<f64> = (0..RECORD).map(|i| if i < 2000 { 0.0 } else { 1.0 }).collect();

    let measured = measure(&record, 1e-6, VOLTS_PER_COUNT);

    assert_eq!(measured.positive_width, Err(Invalid::NoCompletePulse));
    assert_eq!(measured.duty_cycle, Err(Invalid::NoCompletePulse));
}

#[test]
fn given_an_edge_spanning_two_samples_when_measured_then_rise_time_is_too_fast() {
    let record = square(1000, 100, 2, 1.9);

    let measured = measure(&record, 1e-6, VOLTS_PER_COUNT);

    assert_eq!(measured.rise_time, Err(Invalid::EdgeTooFast));
    assert_eq!(measured.fall_time, Err(Invalid::EdgeTooFast));
}

#[test]
fn given_an_edge_of_forty_samples_between_10_and_90_percent_when_measured_then_rise_and_fall_are_250_ns() {
    let record = square(1000, 100, 50, 1.9);

    let measured = measure(&record, 6.25e-9, VOLTS_PER_COUNT);

    assert!(near(measured.rise_time.unwrap(), 250e-9, 0.02));
    assert!(near(measured.fall_time.unwrap(), 250e-9, 0.02));
}

#[test]
fn given_a_rising_edge_that_starts_above_the_ten_percent_level_when_measured_then_no_complete_edge() {
    let mut record = vec![0.3; 5];
    record.extend([0.5, 0.7, 0.9]);
    record.extend(vec![1.0; 50]);
    record.extend(vec![0.0; 100]);

    let measured = measure(&record, 1e-6, VOLTS_PER_COUNT);

    assert_eq!(measured.rise_time, Err(Invalid::NoCompleteEdge));
    assert_eq!(measured.fall_time, Err(Invalid::EdgeTooFast));
}

#[test]
fn given_noise_of_two_counts_on_a_sixty_count_square_when_measured_then_crossings_are_not_doubled() {
    let level = 60.0 * VOLTS_PER_COUNT;
    let mut seed = 12345_u32;
    let record: Vec<f64> = square(1000, 100, 20, level)
        .into_iter()
        .map(|v| {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let counts = f64::from(seed >> 16 & 0x3) - 1.5;
            v + counts * 2.0 / 1.5 * VOLTS_PER_COUNT
        })
        .collect();

    let measured = measure(&record, 1e-6, VOLTS_PER_COUNT);

    assert!(near(measured.frequency.unwrap(), 1000.0, 2e-3));
    assert!((measured.duty_cycle.unwrap() - 50.0).abs() < 2.0);
}

#[test]
fn given_a_triangle_when_measured_then_duty_is_half_and_amplitude_and_frequency_hold() {
    let record = triangle(512, 2.0);

    let measured = measure(&record, 1e-6, VOLTS_PER_COUNT);

    assert!(near(measured.frequency.unwrap(), 1e6 / 512.0, 1e-3));
    assert!((measured.duty_cycle.unwrap() - 50.0).abs() < 1.0);
    assert!(near(measured.vpp, 2.0, 0.01));
    assert!(near(measured.vmean.unwrap(), 1.0, 0.02));
}

#[test]
fn given_a_triangle_when_measured_then_edge_times_follow_the_median_levels() {
    let record = triangle(512, 2.0);

    let measured = measure(&record, 1e-6, VOLTS_PER_COUNT);

    assert!(near(measured.rise_time.unwrap(), 0.4 * 256e-6, 0.02));
    assert!(near(measured.fall_time.unwrap(), 0.4 * 256e-6, 0.02));
}

#[test]
fn given_a_record_that_starts_high_when_measured_then_frequency_fits_from_the_falling_edge_first() {
    let record = square(1250, 1250 - 100, 0, 1.9);
    assert!(record[0] > 0.0);

    let measured = measure(&record, 800e-9, VOLTS_PER_COUNT);

    assert!(near(measured.frequency.unwrap(), 1000.0, 1e-4));
    assert!(near(measured.period.unwrap(), 1e-3, 1e-4));
}

#[test]
fn given_exactly_two_rising_edges_when_measured_then_frequency_comes_from_their_spacing() {
    let record: Vec<f64> = (0..1500).map(|i| if i % 500 < 250 { 0.0 } else { 1.0 }).collect();

    let measured = measure(&record, 1e-6, VOLTS_PER_COUNT);

    assert!(near(measured.frequency.unwrap(), 2000.0, 1e-3));
    assert!(near(measured.period.unwrap(), 500e-6, 1e-3));
}

#[test]
fn given_samples_whose_sum_is_a_rounding_residue_when_measured_then_vmean_is_exactly_zero() {
    let record: Vec<f64> = [0.2, 0.1, -0.3].iter().copied().cycle().take(3000).collect();

    let measured = measure(&record, 1e-6, VOLTS_PER_COUNT);

    assert_eq!(measured.vmean, Ok(0.0));
}
