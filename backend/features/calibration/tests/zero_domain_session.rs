mod common;

use common::RANGES;
use sweepscope_calibration::domain::session::{Accepted, SessionMachine, WrongStep};
use sweepscope_calibration::ZeroStep;

fn session() -> SessionMachine {
    SessionMachine::new(RANGES.to_vec())
}

fn at_channel_one() -> SessionMachine {
    let mut machine = session();
    machine.start().unwrap();
    machine
}

fn offsets(seed: f64) -> Vec<f64> {
    RANGES.iter().map(|range| seed * range).collect()
}

fn accept_channels(machine: &mut SessionMachine, count: usize) -> Option<Accepted> {
    let mut last = None;
    for channel in 0..count {
        machine.measure().unwrap();
        last = Some(machine.accept(offsets(channel as f64 + 1.0)).unwrap());
    }
    last
}

#[test]
fn given_a_new_session_when_viewed_then_it_is_the_intro_without_a_channel() {
    let view = session().view();

    assert_eq!(view.step, ZeroStep::Intro);
    assert_eq!(view.channel, None);
    assert_eq!(view.done_channels, [false; 4]);
}

#[test]
fn given_the_intro_when_calibrate_is_pressed_then_it_is_refused() {
    let mut machine = session();

    assert_eq!(machine.measure(), Err(WrongStep(ZeroStep::Intro)));
    assert_eq!(machine.step(), ZeroStep::Intro);
}

#[test]
fn given_the_intro_when_started_then_channel_one_is_in_turn() {
    let machine = at_channel_one();

    let view = machine.view();
    assert_eq!(view.step, ZeroStep::Channel);
    assert_eq!(view.channel, Some(1));
}

#[test]
fn given_channel_one_when_calibrate_is_pressed_then_it_is_measuring_channel_one_from_zero_progress() {
    let mut machine = at_channel_one();

    let channel = machine.measure().unwrap();

    let view = machine.view();
    assert_eq!(channel, 0);
    assert_eq!(view.step, ZeroStep::Measuring);
    assert_eq!(view.progress, Some(0.0));
}

#[test]
fn given_measuring_when_progress_is_reported_then_it_is_shown_and_kept_within_zero_and_one() {
    let mut machine = at_channel_one();
    machine.measure().unwrap();

    machine.set_progress(0.5);
    assert_eq!(machine.view().progress, Some(0.5));
    machine.set_progress(7.0);
    assert_eq!(machine.view().progress, Some(1.0));
}

#[test]
fn given_an_accepted_channel_when_it_is_not_the_last_then_the_next_channel_is_in_turn_and_nothing_is_committed() {
    let mut machine = at_channel_one();

    let accepted = accept_channels(&mut machine, 1);

    let view = machine.view();
    assert_eq!(accepted, Some(Accepted::NextChannel));
    assert_eq!((view.step, view.channel), (ZeroStep::Channel, Some(2)));
    assert_eq!(view.done_channels, [true, false, false, false]);
    assert_eq!(view.progress, None);
}

#[test]
fn given_three_accepted_channels_when_the_fourth_is_accepted_then_all_four_results_are_handed_out_once() {
    let mut machine = at_channel_one();
    assert_eq!(accept_channels(&mut machine, 3), Some(Accepted::NextChannel));

    machine.measure().unwrap();
    let accepted = machine.accept(offsets(4.0)).unwrap();

    let Accepted::Complete(all) = accepted else {
        panic!("expected Complete")
    };
    assert_eq!(all[0], offsets(1.0));
    assert_eq!(all[3], offsets(4.0));
    assert_eq!(machine.step(), ZeroStep::Measuring);
}

#[test]
fn given_a_completed_session_when_the_commit_succeeds_then_it_is_done_with_a_table_of_twelve_ascending_rows() {
    let mut machine = at_channel_one();
    accept_channels(&mut machine, 4);

    machine.committed().unwrap();

    let view = machine.view();
    let table = view.table.unwrap();
    assert_eq!(view.step, ZeroStep::Done);
    assert_eq!(table.len(), 12);
    let ranges: Vec<f64> = table.iter().map(|row| row.range.value).collect();
    assert!(ranges.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(table[0].range.text, "2");
    assert_eq!(table[0].range.unit, "mV/div");
    assert_eq!(table[0].offsets[0].as_ref().unwrap().value, 0.002);
}

#[test]
fn given_a_channel_with_fewer_offsets_when_done_then_the_missing_cells_are_none() {
    let mut machine = at_channel_one();
    machine.measure().unwrap();
    machine.accept(vec![0.001; 11]).unwrap();
    accept_channels(&mut machine, 3);
    machine.committed().unwrap();

    let table = machine.view().table.unwrap();

    assert!(table[10].offsets[0].is_some());
    assert!(table[11].offsets[0].is_none());
    assert!(table[11].offsets[1].is_some());
}

fn first_cell_after(first_channel: Vec<f64>) -> (String, String) {
    let mut machine = at_channel_one();
    machine.measure().unwrap();
    machine.accept(first_channel).unwrap();
    accept_channels(&mut machine, 3);
    machine.committed().unwrap();
    let cell = machine.view().table.unwrap()[0].offsets[0].clone().unwrap();
    (cell.text, cell.unit)
}

#[test]
fn given_a_stored_zero_when_done_then_the_cell_reads_zero_volts() {
    let cell = first_cell_after(vec![0.0; RANGES.len()]);

    assert_eq!(cell, ("0".to_owned(), "V".to_owned()));
}

#[test]
fn given_a_small_stored_offset_when_done_then_the_cell_keeps_the_engineering_form() {
    let cell = first_cell_after(vec![-41.2e-6; RANGES.len()]);

    assert_eq!(cell, ("\u{2212}41.2".to_owned(), "uV".to_owned()));
}

#[test]
fn given_a_rejected_channel_when_viewed_then_it_keeps_the_channel_and_earlier_results_and_shows_the_lines() {
    let mut machine = at_channel_one();
    accept_channels(&mut machine, 2);
    machine.measure().unwrap();

    machine.reject(vec!["Noise 1 mV".to_owned()], true).unwrap();

    let view = machine.view();
    assert_eq!((view.step, view.channel), (ZeroStep::NotShorted, Some(3)));
    assert_eq!(view.done_channels, [true, true, false, false]);
    assert_eq!(view.not_shorted, Some(vec!["Noise 1 mV".to_owned()]));
}

#[test]
fn given_a_rejected_channel_when_calibrate_is_pressed_again_then_it_measures_the_same_channel_without_the_lines() {
    let mut machine = at_channel_one();
    machine.measure().unwrap();
    machine.reject(vec!["x".to_owned()], true).unwrap();

    let channel = machine.measure().unwrap();

    assert_eq!(channel, 0);
    assert_eq!(machine.view().not_shorted, None);
}

#[test]
fn given_three_accepted_channels_when_cancelled_then_nothing_remains() {
    let mut machine = at_channel_one();
    accept_channels(&mut machine, 3);

    machine.cancel();

    assert_eq!(machine.view().done_channels, [false; 4]);
}

#[test]
fn given_a_session_when_it_fails_then_the_failure_text_is_shown_and_progress_cleared() {
    let mut machine = at_channel_one();
    machine.measure().unwrap();

    machine.fail("The scope was unplugged.");

    let view = machine.view();
    assert_eq!(view.step, ZeroStep::Failed);
    assert_eq!(view.failure.as_deref(), Some("The scope was unplugged."));
    assert_eq!(view.progress, None);
    assert_eq!(view.channel, Some(1));
}

#[test]
fn given_a_session_not_measuring_when_a_result_arrives_then_it_is_refused() {
    let mut machine = at_channel_one();

    assert_eq!(machine.accept(offsets(1.0)), Err(WrongStep(ZeroStep::Channel)));
    assert_eq!(machine.committed(), Err(WrongStep(ZeroStep::Channel)));
    assert_eq!(machine.reject(vec![], true), Err(WrongStep(ZeroStep::Channel)));
}
