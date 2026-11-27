mod common;

use common::RANGES;
use sweepscope_calibration::domain::session::{SessionMachine, WrongStep};
use sweepscope_calibration::ZeroStep;

fn rejected_channel_one(continuable: bool) -> SessionMachine {
    let mut machine = SessionMachine::new(RANGES.to_vec());
    machine.start().unwrap();
    machine.measure().unwrap();
    machine.reject(vec!["x".to_owned()], continuable).unwrap();
    machine
}

fn offsets() -> Vec<f64> {
    vec![0.0; RANGES.len()]
}

#[test]
fn given_a_continuable_rejection_when_viewed_then_continue_is_offered() {
    let machine = rejected_channel_one(true);

    assert!(machine.view().can_continue);
}

#[test]
fn given_a_clipped_rejection_when_viewed_then_continue_is_not_offered_and_resume_is_refused() {
    let mut machine = rejected_channel_one(false);

    assert!(!machine.view().can_continue);
    assert_eq!(machine.resume(), Err(WrongStep(ZeroStep::NotShorted)));
}

#[test]
fn given_a_continuable_rejection_when_resumed_then_it_measures_the_same_channel_and_marks_it_overridden() {
    let mut machine = rejected_channel_one(true);

    machine.resume().unwrap();

    let view = machine.view();
    assert_eq!((view.step, view.channel), (ZeroStep::Measuring, Some(1)));
    assert_eq!(view.not_shorted, None);
    assert!(!view.can_continue);
    assert_eq!(view.overridden, [true, false, false, false]);
}

#[test]
fn given_a_resumed_channel_when_it_is_accepted_then_the_next_channel_follows_and_the_mark_stays() {
    let mut machine = rejected_channel_one(true);
    machine.resume().unwrap();

    machine.accept(offsets()).unwrap();

    let view = machine.view();
    assert_eq!((view.step, view.channel), (ZeroStep::Channel, Some(2)));
    assert_eq!(view.overridden, [true, false, false, false]);
}

#[test]
fn given_an_overridden_channel_when_it_is_measured_again_then_the_mark_is_gone() {
    let mut machine = rejected_channel_one(true);
    machine.resume().unwrap();
    machine.reject(vec!["clipped".to_owned()], false).unwrap();

    machine.measure().unwrap();

    assert_eq!(machine.view().overridden, [false; 4]);
}

#[test]
fn given_a_session_not_at_not_shorted_when_resumed_then_it_is_refused() {
    let mut machine = SessionMachine::new(RANGES.to_vec());
    machine.start().unwrap();

    assert_eq!(machine.resume(), Err(WrongStep(ZeroStep::Channel)));
}
