mod common;

use common::*;
use sweepscope_calibration::domain::statistics::Reading;
use sweepscope_calibration::domain::sweep::{MeasurePlan, RejectionKind, Step, ZeroSweep};

fn plan(step: &Step) -> usize {
    match step {
        Step::Measure(MeasurePlan { range_index }) => *range_index,
        other => panic!("expected a plan, got {other:?}"),
    }
}

/// Answers every plan with `answer(range_index)` until the sweep ends.
fn run(sweep: &mut ZeroSweep, answer: impl Fn(usize) -> Reading) -> (Vec<usize>, Step) {
    let mut plans = Vec::new();
    let mut step = sweep.next(None);
    while let Step::Measure(MeasurePlan { range_index }) = step {
        plans.push(range_index);
        step = sweep.next(Some(&answer(range_index)));
    }
    (plans, step)
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
}

#[test]
fn given_a_new_sweep_when_the_first_plan_is_asked_then_it_is_the_gate_range() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());

    assert_eq!(plan(&sweep.next(None)), GATE_INDEX);
}

#[test]
fn given_a_shorted_gate_reading_when_the_sweep_continues_then_the_most_sensitive_range_is_next() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());
    sweep.next(None);

    let step = sweep.next(Some(&reading(&alternating(128, 1))));

    assert_eq!(plan(&step), 0);
}

#[test]
fn given_twelve_accepted_ranges_when_the_sweep_runs_then_order_is_gate_first_and_then_ascending() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());

    let (plans, step) = run(&mut sweep, |_| reading(&flat(128)));

    assert_eq!(plans, vec![5, 0, 1, 2, 3, 4, 6, 7, 8, 9, 10, 11]);
    assert!(matches!(step, Step::Done(_)));
}

#[test]
fn given_a_sweep_in_progress_when_ranges_are_accepted_then_progress_rises_to_one() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());
    let mut seen = vec![sweep.progress()];
    let mut step = sweep.next(None);

    while let Step::Measure(_) = step {
        step = sweep.next(Some(&reading(&flat(128))));
        seen.push(sweep.progress());
    }

    assert!(seen.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(seen.first(), Some(&0.0));
    assert_eq!(seen.last(), Some(&1.0));
}

#[test]
fn given_a_mean_of_eight_codes_at_two_millivolts_when_accepted_then_the_offset_is_0_662_millivolts() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());

    let (_, step) = run(&mut sweep, |index| reading(&flat(if index == 0 { 136 } else { 128 })));

    let Step::Done(offsets) = step else {
        panic!("expected Done")
    };
    assert_eq!(offsets.len(), 12);
    assert_close(offsets[0], 8.0 * 0.002 / COUNTS);
    assert!((offsets[0] - 0.000_662).abs() < 1e-6);
    assert_close(offsets[1], 0.0);
}

#[test]
fn given_a_mean_below_zero_when_accepted_then_the_offset_is_negative() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());

    let (_, step) = run(&mut sweep, |index| reading(&flat(if index == 2 { 126 } else { 128 })));

    let Step::Done(offsets) = step else {
        panic!("expected Done")
    };
    assert_close(offsets[2], -2.0 * 0.01 / COUNTS);
}

#[test]
fn given_twelve_codes_peak_to_peak_at_the_gate_when_judged_then_a_signal_is_present_with_volts_and_limit() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());
    sweep.next(None);

    let step = sweep.next(Some(&reading(&alternating(128, 6))));

    let Step::NotShorted(rejection) = step else {
        panic!("expected NotShorted")
    };
    assert_eq!(rejection.kind, RejectionKind::SignalPresent);
    assert_eq!(rejection.range_index, GATE_INDEX);
    assert_eq!(rejection.line, "Peak to peak 49.7 mV at 100 mV/div, limit 24.8 mV.");
}

#[test]
fn given_a_standard_deviation_above_the_gate_limit_when_judged_then_a_signal_is_present() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());
    sweep.next(None);

    let step = sweep.next(Some(&reading(&alternating(128, 2))));

    let Step::NotShorted(rejection) = step else {
        panic!("expected NotShorted")
    };
    assert_eq!(rejection.kind, RejectionKind::SignalPresent);
    assert!(rejection.line.starts_with("Noise "), "{}", rejection.line);
}

#[test]
fn given_a_mean_nine_codes_above_zero_at_the_gate_when_judged_then_the_level_is_off_zero() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());
    sweep.next(None);

    let step = sweep.next(Some(&reading(&flat(137))));

    let Step::NotShorted(rejection) = step else {
        panic!("expected NotShorted")
    };
    assert_eq!(rejection.kind, RejectionKind::LevelOffZero);
    assert_eq!(rejection.line, "Level 37.3 mV from zero at 100 mV/div, limit 33.1 mV.");
}

#[test]
fn given_a_mean_of_eight_codes_at_the_gate_when_judged_then_it_is_accepted() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());
    sweep.next(None);

    let step = sweep.next(Some(&reading(&flat(136))));

    assert!(matches!(step, Step::Measure(_)));
}

#[test]
fn given_a_clipped_sample_at_the_gate_when_judged_then_a_signal_is_present() {
    let mut codes = flat(128);
    codes[10] = 255;
    let mut sweep = ZeroSweep::new(RANGES.to_vec());
    sweep.next(None);

    let step = sweep.next(Some(&reading(&codes)));

    let Step::NotShorted(rejection) = step else {
        panic!("expected NotShorted")
    };
    assert_eq!(rejection.kind, RejectionKind::SignalPresent);
    assert_eq!(rejection.line, "Input clipped at 100 mV/div.");
}

#[test]
fn given_a_noisy_later_range_when_judged_then_it_is_too_noisy_and_no_offsets_are_given() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());

    let (_, step) = run(&mut sweep, |index| {
        reading(&if index == 4 { alternating(128, 5) } else { flat(128) })
    });

    let Step::NotShorted(rejection) = step else {
        panic!("expected NotShorted")
    };
    assert_eq!(rejection.kind, RejectionKind::TooNoisy);
    assert_eq!(rejection.range_index, 4);
    assert!(rejection.line.starts_with("Noise "), "{}", rejection.line);
    assert!(rejection.line.contains("rms at 50 mV/div, limit"), "{}", rejection.line);
}

#[test]
fn given_the_noise_of_a_quiet_input_at_two_millivolts_when_judged_then_it_is_accepted() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());

    let (_, step) = run(&mut sweep, |index| {
        reading(&if index == 0 { alternating(130, 24) } else { flat(128) })
    });

    assert!(matches!(step, Step::Done(_)));
}

#[test]
fn given_a_level_thirteen_codes_from_zero_on_another_range_when_judged_then_the_level_is_off_zero() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());

    let (_, step) = run(&mut sweep, |index| reading(&flat(if index == 3 { 141 } else { 128 })));

    let Step::NotShorted(rejection) = step else {
        panic!("expected NotShorted")
    };
    assert_eq!(rejection.kind, RejectionKind::LevelOffZero);
    assert_eq!(rejection.range_index, 3);
}
