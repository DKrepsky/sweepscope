use std::time::Duration;

use sweepscope_acquisition::{next_status, CaptureMeter, Phase, StatusEvent};
use sweepscope_kernel::testing::FakeClock;
use sweepscope_kernel::Clock;

const FRAME_EVERY: Duration = Duration::from_millis(50);

fn run_frames(meter: &mut CaptureMeter, clock: &FakeClock, count: usize, every: Duration) {
    for _ in 0..count {
        meter.record(clock.now());
        clock.sleep(every);
    }
}

#[test]
fn given_a_stopped_scope_when_run_then_the_phase_is_armed() {
    assert_eq!(next_status(Phase::Stopped, StatusEvent::Run), Phase::Armed);
}

#[test]
fn given_a_running_scope_when_a_triggered_frame_arrives_then_the_phase_is_triggered() {
    assert_eq!(next_status(Phase::Armed, StatusEvent::Triggered), Phase::Triggered);
    assert_eq!(
        next_status(Phase::WaitingForTrigger, StatusEvent::Triggered),
        Phase::Triggered
    );
}

#[test]
fn given_a_running_scope_when_an_auto_frame_arrives_then_the_phase_is_auto() {
    assert_eq!(next_status(Phase::Armed, StatusEvent::Auto), Phase::Auto);
    assert_eq!(next_status(Phase::Triggered, StatusEvent::Auto), Phase::Auto);
}

#[test]
fn given_a_running_scope_when_a_capture_times_out_then_the_phase_is_waiting_for_trigger() {
    assert_eq!(
        next_status(Phase::Armed, StatusEvent::TimedOut),
        Phase::WaitingForTrigger
    );
    assert_eq!(
        next_status(Phase::Triggered, StatusEvent::TimedOut),
        Phase::WaitingForTrigger
    );
}

#[test]
fn given_any_running_phase_when_stopped_or_lost_then_the_phase_is_stopped() {
    for phase in [Phase::Armed, Phase::WaitingForTrigger, Phase::Triggered, Phase::Auto] {
        assert_eq!(next_status(phase, StatusEvent::Stop), Phase::Stopped);
        assert_eq!(next_status(phase, StatusEvent::Lost), Phase::Stopped);
    }
}

#[test]
fn given_a_stopped_scope_when_a_late_capture_outcome_arrives_then_the_phase_stays_stopped() {
    for event in [StatusEvent::Triggered, StatusEvent::Auto, StatusEvent::TimedOut] {
        assert_eq!(next_status(Phase::Stopped, event), Phase::Stopped);
    }
}

#[test]
fn given_frames_every_50_ms_when_the_rate_is_read_then_it_is_20_per_second() {
    let clock = FakeClock::default();
    let mut meter = CaptureMeter::default();

    run_frames(&mut meter, &clock, 100, FRAME_EVERY);

    let rate = meter.rate(clock.now()).unwrap();
    assert!((rate - 20.0).abs() < 0.01, "{rate}");
}

#[test]
fn given_a_single_frame_when_the_rate_is_read_then_there_is_no_rate() {
    let clock = FakeClock::default();
    let mut meter = CaptureMeter::default();

    run_frames(&mut meter, &clock, 1, FRAME_EVERY);

    assert_eq!(meter.rate(clock.now()), None);
}

#[test]
fn given_no_frame_for_two_seconds_when_the_rate_is_read_then_there_is_no_rate() {
    let clock = FakeClock::default();
    let mut meter = CaptureMeter::default();
    run_frames(&mut meter, &clock, 10, FRAME_EVERY);

    clock.advance(Duration::from_millis(2100));

    assert_eq!(meter.rate(clock.now()), None);
}

#[test]
fn given_a_cleared_meter_when_the_rate_is_read_then_there_is_no_rate() {
    let clock = FakeClock::default();
    let mut meter = CaptureMeter::default();
    run_frames(&mut meter, &clock, 10, FRAME_EVERY);

    meter.clear();

    assert_eq!(meter.rate(clock.now()), None);
}

#[test]
fn given_20_frames_per_second_when_the_record_lasts_10_ms_then_the_dead_time_is_80_percent() {
    let clock = FakeClock::default();
    let mut meter = CaptureMeter::default();
    run_frames(&mut meter, &clock, 100, FRAME_EVERY);

    let dead = meter.dead_time_percent(clock.now(), 0.010).unwrap();

    assert!((dead - 80.0).abs() < 0.1, "{dead}");
}

#[test]
fn given_a_record_longer_than_the_frame_period_when_the_dead_time_is_read_then_it_is_clamped_to_zero() {
    let clock = FakeClock::default();
    let mut meter = CaptureMeter::default();
    run_frames(&mut meter, &clock, 100, FRAME_EVERY);

    assert_eq!(meter.dead_time_percent(clock.now(), 1.0), Some(0.0));
}

#[test]
fn given_a_negligible_record_when_the_dead_time_is_read_then_it_is_at_most_100_percent() {
    let clock = FakeClock::default();
    let mut meter = CaptureMeter::default();
    run_frames(&mut meter, &clock, 100, FRAME_EVERY);

    assert_eq!(meter.dead_time_percent(clock.now(), 0.0), Some(100.0));
}

#[test]
fn given_a_stopped_scope_when_a_single_is_armed_then_the_phase_waits_for_a_trigger() {
    assert_eq!(
        next_status(Phase::Stopped, StatusEvent::ArmSingle),
        Phase::WaitingForTrigger
    );
}

#[test]
fn given_a_waiting_single_when_it_is_done_then_the_phase_is_done() {
    assert_eq!(
        next_status(Phase::WaitingForTrigger, StatusEvent::SingleDone),
        Phase::Done
    );
}

#[test]
fn given_a_done_single_when_run_or_stop_then_the_phase_is_armed_or_stopped() {
    assert_eq!(next_status(Phase::Done, StatusEvent::Run), Phase::Armed);
    assert_eq!(next_status(Phase::Done, StatusEvent::Stop), Phase::Stopped);
}
