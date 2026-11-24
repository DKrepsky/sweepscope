mod zero_module_support;

use sweepscope_calibration::domain::offsets::OffsetFile;
use sweepscope_calibration::ZeroStep;
use zero_module_support::{open_input, shorted, Rig, COUNTS, RANGES, UNIT};

const GATE_RANGE_INDEX: u8 = 5;

fn stored_volts(rig: &Rig) -> Option<[Vec<f64>; 4]> {
    let file = OffsetFile::load(&rig.storage).unwrap();
    file.unit(UNIT).map(|unit| unit.volts.clone())
}

fn connected_rig() -> Rig {
    let rig = Rig::new();
    rig.connect(Some(UNIT));
    rig
}

#[test]
fn given_a_ready_scope_with_a_serial_when_connected_then_calibration_is_available_and_uncalibrated() {
    let rig = connected_rig();

    let state = rig.state();

    assert!(state.availability.enabled);
    assert!(!state.unit.unwrap().calibrated);
}

#[test]
fn given_an_available_scope_when_the_dialog_is_opened_then_the_instrument_is_held_and_the_intro_shows() {
    let rig = connected_rig();

    rig.open().unwrap();

    assert_eq!(rig.acquisition.hold_requests(), vec![true]);
    assert_eq!(rig.step(), Some(ZeroStep::Intro));
}

#[test]
fn given_a_started_session_when_a_channel_is_measured_then_it_ends_at_the_next_channel_with_progress_done() {
    let rig = connected_rig();
    rig.open().unwrap();
    rig.start().unwrap();

    rig.measure().unwrap();
    let state = rig.until_step(ZeroStep::Channel);

    let session = state.session.unwrap();
    assert_eq!(session.channel, Some(2));
    assert_eq!(session.done_channels, [true, false, false, false]);
}

#[test]
fn given_a_channel_when_it_is_measured_then_the_gate_range_goes_first_and_the_rest_ascend() {
    let rig = connected_rig();
    rig.open().unwrap();
    rig.start().unwrap();

    rig.measure().unwrap();
    rig.until_step(ZeroStep::Channel);

    let ranges: Vec<u8> = rig
        .acquisition
        .probe_requests()
        .iter()
        .map(|probe| probe.setup.channels[0].range_index)
        .collect();
    let mut expected = vec![GATE_RANGE_INDEX];
    expected.extend((0..RANGES.len() as u8).filter(|&i| i != GATE_RANGE_INDEX));
    assert_eq!(ranges, expected);
}

#[test]
fn given_a_channel_in_turn_when_it_is_measured_then_that_channel_is_the_trigger_source_of_every_probe() {
    let rig = connected_rig();
    rig.open().unwrap();
    rig.start().unwrap();
    rig.measure().unwrap();
    rig.until_step(ZeroStep::Channel);

    rig.measure().unwrap();
    rig.until_step(ZeroStep::Channel);

    let sources: Vec<u8> = rig
        .acquisition
        .probe_requests()
        .iter()
        .map(|probe| probe.setup.trigger.source)
        .collect();
    assert_eq!(sources[..12], [0; 12]);
    assert_eq!(sources[12..], [1; 12]);
}

#[test]
fn given_three_finished_channels_when_the_fourth_is_not_yet_done_then_nothing_is_stored_or_announced() {
    let rig = connected_rig();

    rig.calibrate_channels(3);

    assert_eq!(stored_volts(&rig), None);
    assert!(rig.offsets.events().iter().all(|e| e.offsets.is_none()));
}

#[test]
fn given_four_shorted_channels_when_the_last_is_measured_then_the_session_is_done_with_a_table() {
    let rig = connected_rig();

    rig.calibrate_channels(4);

    let table = rig.state().session.unwrap().table.unwrap();
    assert_eq!(table.len(), RANGES.len());
    assert!(table.iter().all(|row| row.offsets.iter().all(Option::is_some)));
}

#[test]
fn given_four_shorted_channels_when_done_then_volts_are_codes_times_range_over_counts_in_the_file() {
    let rig = connected_rig();

    rig.calibrate_channels(4);

    let volts = stored_volts(&rig).unwrap();
    for channel in &volts {
        assert!((channel[0] - 8.0 * RANGES[0] / COUNTS).abs() < 1e-12);
        assert!((channel[11] - 1.0 * RANGES[11] / COUNTS).abs() < 1e-12);
    }
}

#[test]
fn given_four_shorted_channels_when_done_then_the_unit_is_stamped_and_calibrated() {
    let rig = connected_rig();

    rig.calibrate_channels(4);

    let unit = rig.state().unit.unwrap();
    assert!(unit.calibrated);
    assert_eq!(unit.stored_at_utc.as_deref(), Some("2023-11-14T22:13:20Z"));
}

#[test]
fn given_a_commit_when_it_is_written_then_the_offsets_are_announced_for_the_unit() {
    let rig = connected_rig();

    rig.calibrate_channels(4);

    let announced = rig.offsets.events().into_iter().last().unwrap().offsets.unwrap();
    assert_eq!(announced.unit_id, UNIT);
    assert_eq!(Some(announced.volts), stored_volts(&rig));
}

#[test]
fn given_a_done_session_when_it_is_closed_then_the_hold_is_released_and_the_session_is_gone() {
    let rig = connected_rig();
    rig.calibrate_channels(4);

    rig.close().unwrap();

    assert_eq!(rig.acquisition.hold_requests(), vec![true, false]);
    assert_eq!(rig.state().session, None);
}

#[test]
fn given_an_open_input_when_a_channel_is_measured_then_it_stays_on_the_channel_with_the_reason() {
    let rig = connected_rig();
    rig.set_signal(open_input);
    rig.open().unwrap();
    rig.start().unwrap();

    rig.measure().unwrap();
    let session = rig.until_step(ZeroStep::NotShorted).session.unwrap();

    assert_eq!(session.channel, Some(1));
    assert!(session.not_shorted.unwrap()[0].starts_with("Peak to peak"));
}

#[test]
fn given_a_not_shorted_channel_when_it_is_shorted_and_measured_again_then_it_is_accepted() {
    let rig = connected_rig();
    rig.set_signal(open_input);
    rig.open().unwrap();
    rig.start().unwrap();
    rig.measure().unwrap();
    rig.until_step(ZeroStep::NotShorted);

    rig.set_signal(shorted);
    rig.measure().unwrap();
    let session = rig.until_step(ZeroStep::Channel).session.unwrap();

    assert_eq!(session.channel, Some(2));
    assert_eq!(session.not_shorted, None);
}

#[test]
fn given_a_session_at_channel_three_when_it_is_closed_then_nothing_is_written_and_the_hold_is_released() {
    let rig = connected_rig();
    rig.calibrate_channels(2);

    rig.close().unwrap();

    assert_eq!(stored_volts(&rig), None);
    assert_eq!(rig.acquisition.hold_requests(), vec![true, false]);
    assert!(rig.offsets.events().iter().all(|e| e.offsets.is_none()));
}

#[test]
fn given_a_session_closed_while_measuring_when_a_late_answer_arrives_then_it_is_ignored() {
    let rig = connected_rig();
    rig.acquisition.behaviour.lock().unwrap().answers = false;
    rig.open().unwrap();
    rig.start().unwrap();
    rig.measure().unwrap();
    let late = rig.acquisition.probe_requests()[0].id;

    rig.close().unwrap();
    rig.bus.publish(sweepscope_kernel::contracts::ProbeCaptured {
        id: late,
        frames: vec![],
    });
    rig.disconnect();
    let state = rig.until(|s| !s.availability.enabled);

    assert_eq!(state.session, None);
}

#[test]
fn given_a_stored_unit_when_the_same_unit_connects_then_its_offsets_are_announced_and_it_is_calibrated() {
    let rig = Rig::new();
    rig.connect(Some(UNIT));
    rig.calibrate_channels(4);
    let stored = stored_volts(&rig);
    rig.close().unwrap();
    rig.disconnect();
    rig.until(|s| !s.availability.enabled);

    rig.connect(Some(UNIT));

    let announced = rig.offsets.events().into_iter().last().unwrap().offsets.unwrap();
    assert_eq!(Some(announced.volts), stored);
    assert!(rig.state().unit.unwrap().calibrated);
}

#[test]
fn given_a_stored_unit_when_another_unit_connects_then_no_offsets_are_announced_for_it() {
    let rig = Rig::new();
    rig.connect(Some(UNIT));
    rig.calibrate_channels(4);
    rig.close().unwrap();
    rig.disconnect();
    rig.until(|s| !s.availability.enabled);

    rig.connect(Some("6254BC/pcb/0002"));

    assert_eq!(rig.offsets.events().into_iter().last().unwrap().offsets, None);
    assert!(!rig.state().unit.unwrap().calibrated);
}

#[test]
fn given_a_calibrated_unit_when_it_is_calibrated_again_then_the_new_values_replace_the_old() {
    let rig = connected_rig();
    rig.calibrate_channels(4);
    rig.close().unwrap();
    rig.set_signal(|_, _| vec![128; 300]);

    rig.calibrate_channels(4);

    let volts = stored_volts(&rig).unwrap();
    assert!(volts.iter().all(|channel| channel.iter().all(|&v| v == 0.0)));
}
