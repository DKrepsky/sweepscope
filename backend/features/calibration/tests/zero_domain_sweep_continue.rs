mod common;

use common::*;
use sweepscope_calibration::domain::statistics::Reading;
use sweepscope_calibration::domain::sweep::{MeasurePlan, NothingPending, RejectionKind, Step, ZeroSweep};

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

/// A noiseless reading whose mean sits `codes` above the zero code.
fn level(codes: u8) -> Reading {
    reading(&flat(128 + codes))
}

/// Gate range clean, then the most sensitive range reads `codes` above zero.
fn rejected_at_two_millivolts(sweep: &mut ZeroSweep, codes: u8) -> Step {
    sweep.next(None);
    sweep.next(Some(&level(0)));
    sweep.next(Some(&level(codes)))
}

fn accepted_at_two_millivolts(codes: u8) -> ZeroSweep {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());
    rejected_at_two_millivolts(&mut sweep, codes);
    sweep.accept_rejected().unwrap();
    sweep
}

fn rejection(step: Step) -> sweepscope_calibration::domain::sweep::Rejection {
    match step {
        Step::NotShorted(rejection) => rejection,
        other => panic!("expected NotShorted, got {other:?}"),
    }
}

#[test]
fn given_a_level_of_1_45_millivolts_at_two_millivolts_when_judged_then_it_is_accepted() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());

    let step = rejected_at_two_millivolts(&mut sweep, 17);

    assert_eq!(plan(&step), 1);
}

#[test]
fn given_a_level_of_three_millivolts_at_two_millivolts_when_judged_then_the_limit_is_2_50_millivolts() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());

    let found = rejection(rejected_at_two_millivolts(&mut sweep, 36));

    assert_eq!(found.kind, RejectionKind::LevelOffZero);
    assert_eq!(found.range_index, 0);
    assert_eq!(found.line, "Level 2.98 mV from zero at 2 mV/div, limit 2.50 mV.");
    assert!(found.continuable);
}

#[test]
fn given_six_millivolts_at_five_millivolts_when_judged_then_the_twelve_code_limit_applies() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());

    let (_, step) = run(&mut sweep, |index| level(if index == 1 { 29 } else { 0 }));

    let found = rejection(step);
    assert_eq!(found.range_index, 1);
    assert!(found.line.ends_with("limit 2.50 mV."), "{}", found.line);
}

#[test]
fn given_a_dc_level_of_20_millivolts_when_judged_then_the_gate_accepts_it_and_two_millivolts_rejects_it() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());
    sweep.next(None);

    let at_gate = sweep.next(Some(&level(5)));
    let found = rejection(sweep.next(Some(&reading(&flat(255)))));

    assert_eq!(plan(&at_gate), 0);
    assert_eq!(found.range_index, 0);
}

#[test]
fn given_a_clipped_gate_reading_when_rejected_then_it_is_not_continuable() {
    let mut codes = flat(128);
    codes[10] = 255;
    let mut sweep = ZeroSweep::new(RANGES.to_vec());
    sweep.next(None);

    let found = rejection(sweep.next(Some(&reading(&codes))));

    assert!(!found.continuable);
}

#[test]
fn given_a_clipped_reading_off_zero_at_two_millivolts_when_rejected_then_it_is_continuable() {
    let mut codes = flat(128 + 36);
    codes[3] = 0;
    let mut sweep = ZeroSweep::new(RANGES.to_vec());
    sweep.next(None);
    sweep.next(Some(&level(0)));

    let found = rejection(sweep.next(Some(&reading(&codes))));

    assert_eq!(found.kind, RejectionKind::LevelOffZero);
    assert_eq!(found.range_index, 0);
    assert!(found.continuable);
}

#[test]
fn given_a_noisy_unclipped_gate_reading_when_rejected_then_it_is_continuable() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());
    sweep.next(None);

    let found = rejection(sweep.next(Some(&reading(&alternating(128, 6)))));

    assert!(found.continuable);
}

#[test]
fn given_a_rejection_when_it_is_accepted_then_its_offset_is_stored_and_the_next_range_follows() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());
    rejected_at_two_millivolts(&mut sweep, 36);

    let step = sweep.accept_rejected().unwrap();

    assert_eq!(plan(&step), 1);
    let (_, done) = run(&mut sweep, |_| level(0));
    let Step::Done(offsets) = done else {
        panic!("expected Done")
    };
    assert!((offsets[0] - 36.0 * RANGES[0] / COUNTS).abs() < 1e-12);
}

#[test]
fn given_an_accepted_rejection_when_later_ranges_are_noisy_or_off_zero_then_they_are_accepted() {
    let mut sweep = accepted_at_two_millivolts(36);

    let (plans, step) = run(&mut sweep, |index| {
        if index % 2 == 0 {
            level(60)
        } else {
            reading(&alternating(128, 30))
        }
    });

    assert_eq!(plans, vec![1, 2, 3, 4, 6, 7, 8, 9, 10, 11]);
    assert!(matches!(step, Step::Done(_)));
}

/// 2 mV/div on a perfect short at position 0: zero code 35, noise about 24 codes, low samples clip at code 0.
fn clipped_low_at_two_millivolts() -> Reading {
    let codes: Vec<u8> = (0..300).map(|i| [0, 35, 70][i % 3]).collect();
    Reading::new(&codes, 35.0, COUNTS).unwrap()
}

#[test]
fn given_an_accepted_rejection_when_clipped_low_samples_come_at_two_millivolts_then_the_channel_completes() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());
    sweep.next(None);
    rejection(sweep.next(Some(&reading(&alternating(128, 6)))));
    sweep.accept_rejected().unwrap();

    let (plans, step) = run(&mut sweep, |index| {
        if index == 0 {
            clipped_low_at_two_millivolts()
        } else {
            level(0)
        }
    });

    assert_eq!(plans[0], 0);
    assert!(matches!(step, Step::Done(_)));
}

#[test]
fn given_a_normal_sweep_when_clipped_low_samples_come_at_two_millivolts_then_they_are_accepted() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());

    let (_, step) = run(&mut sweep, |index| {
        if index == 0 {
            clipped_low_at_two_millivolts()
        } else {
            level(0)
        }
    });

    assert!(matches!(step, Step::Done(_)));
}

#[test]
fn given_an_accepted_rejection_when_the_sweep_goes_on_then_the_gate_is_not_measured_again() {
    let mut sweep = accepted_at_two_millivolts(36);

    let (plans, _) = run(&mut sweep, |_| level(0));

    assert!(!plans.contains(&GATE_INDEX));
}

#[test]
fn given_no_rejection_when_a_rejected_reading_is_accepted_then_it_is_refused() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());
    sweep.next(None);

    assert_eq!(sweep.accept_rejected(), Err(NothingPending));
}

#[test]
fn given_an_accepted_rejection_when_it_is_accepted_again_then_it_is_refused() {
    let mut sweep = accepted_at_two_millivolts(36);

    assert_eq!(sweep.accept_rejected(), Err(NothingPending));
}

#[test]
fn given_a_rejection_when_the_progress_is_read_then_it_has_not_moved() {
    let mut sweep = ZeroSweep::new(RANGES.to_vec());
    sweep.next(None);
    sweep.next(Some(&level(0)));
    let before = sweep.progress();

    sweep.next(Some(&level(36)));

    assert_eq!(sweep.progress(), before);
}
