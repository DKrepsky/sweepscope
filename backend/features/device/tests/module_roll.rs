//! Roll loop of the device worker: indexed samples, gaps, restarts and the end of a roll.
mod module_support;

use std::sync::atomic::Ordering;
use std::time::Duration;

use sweepscope_device::Sweep;
use sweepscope_kernel::contracts::{ApplySetup, ConnectionState, RollPolled, RollPolledItem, SetCapturing};

use module_support::*;

fn rolling_rig(source: &mut Source) -> Rig {
    let rig = rig(None, Some(source));
    rig.wait_for_state(ConnectionState::Ready);
    apply(&rig, 1, TB_500MS);
    rig
}

fn apply(rig: &Rig, revision: u64, time_base_index: u8) {
    rig.bus
        .request(ApplySetup {
            setup_revision: revision,
            setup: roll_setup(time_base_index),
        })
        .unwrap();
}

fn capturing(rig: &Rig, on: bool) {
    rig.bus.request(SetCapturing { on }).unwrap();
}

fn next_polled(rig: &Rig) -> RollPolled {
    rig.polled.recv_timeout(WAIT).expect("a roll poll is published")
}

fn next_samples(rig: &Rig) -> (u64, usize) {
    loop {
        if let RollPolledItem::Samples { first_index, frame } = next_polled(rig).item {
            let len = frame.channels[0].as_ref().unwrap().codes.len();
            return (first_index, len);
        }
    }
}

fn calls(source: &Source) -> Vec<&'static str> {
    source.hooks.calls.lock().unwrap().clone()
}

#[test]
fn given_a_streaming_setup_when_capturing_starts_then_samples_arrive_with_contiguous_first_indices() {
    let mut simulator = Source::plugged();
    let rig = rolling_rig(&mut simulator);

    capturing(&rig, true);

    let (first, first_len) = next_samples(&rig);
    let (second, second_len) = next_samples(&rig);
    let (third, _) = next_samples(&rig);
    assert_eq!(first, 0);
    assert_eq!(second, first_len as u64);
    assert_eq!(third, (first_len + second_len) as u64);
}

#[test]
fn given_a_streaming_setup_when_capturing_then_no_capture_is_completed_and_the_revision_is_carried() {
    let mut simulator = Source::plugged();
    let rig = rolling_rig(&mut simulator);

    capturing(&rig, true);

    assert_eq!(next_polled(&rig).setup_revision, 1);
    assert!(rig.completed.try_next().is_none());
}

#[test]
fn given_an_injected_overrun_when_polling_then_a_gap_follows_the_last_index_and_samples_resume_after_it() {
    let mut simulator = Source::plugged();
    let rig = rolling_rig(&mut simulator);
    capturing(&rig, true);
    let (first, len) = next_samples(&rig);
    simulator.control.inject_overrun();

    let (after, missed) = loop {
        if let RollPolledItem::Gap { after, missed } = next_polled(&rig).item {
            break (after, missed);
        }
    };
    let (resumed, _) = next_samples(&rig);

    assert!(after >= first + len as u64 - 1);
    // 50 ms poll interval over a 2 ms sample interval is about 25 samples.
    assert!((10..=100).contains(&missed), "{missed}");
    assert_eq!(resumed, after + 1 + missed);
}

#[test]
fn given_a_roll_when_the_setup_changes_then_the_worker_stops_configures_and_restarts_with_the_new_revision() {
    let mut simulator = Source::plugged();
    let rig = rolling_rig(&mut simulator);
    capturing(&rig, true);
    next_samples(&rig);

    apply(&rig, 2, TB_500MS + 1);

    assert!(wait_until(|| calls(&simulator).len() >= 5));
    assert_eq!(
        calls(&simulator)[..5],
        ["configure", "start_roll", "stop", "configure", "start_roll"]
    );
    let restarted = std::iter::from_fn(|| rig.polled.recv_timeout(WAIT)).find(|polled| polled.setup_revision == 2);
    assert!(matches!(
        restarted.unwrap().item,
        RollPolledItem::Samples { first_index: 0, .. }
    ));
}

#[test]
fn given_a_roll_when_capturing_is_switched_off_then_the_device_stops_and_nothing_more_is_published() {
    let mut simulator = Source::plugged();
    let rig = rolling_rig(&mut simulator);
    capturing(&rig, true);
    next_samples(&rig);

    capturing(&rig, false);

    assert!(wait_until(|| calls(&simulator).last() == Some(&"stop")));
    while rig.polled.try_next().is_some() {}
    assert!(rig.polled.recv_timeout(Duration::from_millis(200)).is_none());
}

#[test]
fn given_a_stopped_roll_when_capturing_resumes_then_the_roll_restarts_at_index_zero_after_configuring_again() {
    let mut simulator = Source::plugged();
    let rig = rolling_rig(&mut simulator);
    capturing(&rig, true);
    next_samples(&rig);
    capturing(&rig, false);
    assert!(wait_until(|| calls(&simulator).last() == Some(&"stop")));
    while rig.polled.try_next().is_some() {}

    capturing(&rig, true);

    assert_eq!(next_samples(&rig).0, 0);
    assert_eq!(calls(&simulator)[2..5], ["stop", "stop", "configure"]);
    assert_eq!(calls(&simulator)[5], "start_roll");
}

#[test]
fn given_a_roll_when_the_device_is_unplugged_then_the_state_is_disconnected_and_the_worker_ends() {
    let mut simulator = Source::plugged();
    let rig = rolling_rig(&mut simulator);
    capturing(&rig, true);
    next_samples(&rig);

    simulator.control.unplug();

    rig.wait_for_state(ConnectionState::Disconnected);
    assert!(wait_until(|| simulator.hooks.live_devices.load(Ordering::SeqCst) == 0));
}

#[test]
fn given_a_frame_setup_when_capturing_then_no_roll_item_is_published() {
    let mut simulator = Source::plugged();
    let rig = rig(None, Some(&mut simulator));
    rig.wait_for_state(ConnectionState::Ready);
    rig.bus
        .request(ApplySetup {
            setup_revision: 1,
            setup: setup(Sweep::Auto, 0.0),
        })
        .unwrap();

    capturing(&rig, true);

    assert!(rig.completed.recv_timeout(WAIT).is_some());
    assert!(rig.polled.try_next().is_none());
}
