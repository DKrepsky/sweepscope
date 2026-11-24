mod autoset_support;
#[path = "domain_support/mod.rs"]
mod domain_support;
mod module_support;

use std::sync::Arc;
use std::time::Duration;

use autoset_support::{capture, Input, Signal};
use domain_support::{capabilities, patch_enabled};
use measurements::measure;
use module_support::{expect_recoverable, ready_rig, DeviceCall, Rig};
use sweepscope_acquisition::{
    ArmSingle, AutosetOutcomeKind, AutosetStatus, ChannelPatch, SetChannel, SetRunning, StartAutoset,
};
use sweepscope_kernel::contracts::{AcquisitionSetup, CaptureOutcome, Sweep};

const FREQUENCIES_HZ: [f64; 7] = [10.0, 100.0, 1e3, 10e3, 100e3, 1e6, 10e6];
const FAILED: &str = "Auto-setup could not find a stable setting. Previous setup kept.";
const RUNNING: &str = "Auto-setup is running";

fn rig_on(signal: Signal) -> (Rig, Input) {
    let rig = ready_rig();
    let input = Input::new(signal);
    let source = input.clone();
    rig.device.respond_with(move |setup| Some(source.frame(setup)));
    (rig, input)
}

/// A scope that never answers: the run stays in its first probe.
fn silent_rig() -> Rig {
    ready_rig()
}

fn status(rig: &Rig) -> AutosetStatus {
    rig.status
        .events()
        .last()
        .map_or_else(AutosetStatus::default, |event| event.0.autoset.clone())
}

fn wait_for_outcome(rig: &Rig) -> AutosetStatus {
    rig.wait(|| status(rig).outcome.is_some() && !status(rig).running);
    status(rig)
}

fn run(rig: &Rig) -> AutosetStatus {
    rig.bus.request(StartAutoset).unwrap();
    wait_for_outcome(rig)
}

fn last_setup(rig: &Rig) -> AcquisitionSetup {
    rig.device.applies().last().unwrap().1.clone()
}

fn seconds_per_div(setup: &AcquisitionSetup) -> f64 {
    capabilities().time_bases[usize::from(setup.time_base_index)].seconds_per_div
}

fn volts_per_div(setup: &AcquisitionSetup) -> f64 {
    capabilities().vertical_ranges[usize::from(setup.channels[0].range_index)].volts_per_div
}

fn measured_screen_periods(signal: Signal, setup: &AcquisitionSetup) -> f64 {
    let frame = capture(signal, setup);
    let data = frame.channels[0].as_ref().unwrap();
    let volts_per_count = data.volts_per_div / frame.counts_per_div;
    let volts: Vec<f64> = data
        .codes
        .iter()
        .map(|c| (f64::from(*c) - data.zero_code) * volts_per_count)
        .collect();
    let period = measure(&volts, frame.sample_interval_s, volts_per_count)
        .period
        .unwrap();
    10.0 * seconds_per_div(setup) / period
}

#[test]
fn given_the_seven_frequencies_when_autoset_runs_then_the_final_setup_shows_2_to_6_periods_and_3_to_7_divisions() {
    for frequency_hz in FREQUENCIES_HZ {
        let signal = Signal::sine(frequency_hz, 1.0);
        let (rig, _) = rig_on(signal);

        let outcome = run(&rig).outcome.unwrap();

        assert_eq!(
            outcome.kind,
            AutosetOutcomeKind::Done,
            "{frequency_hz} Hz: {}",
            outcome.text
        );
        let setup = last_setup(&rig);
        let periods = measured_screen_periods(signal, &setup);
        assert!((2.0..=6.0).contains(&periods), "{frequency_hz} Hz: {periods} periods");
        let divisions = signal.peak_to_peak_volts / volts_per_div(&setup);
        assert!(
            (3.0..=7.0).contains(&divisions),
            "{frequency_hz} Hz: {divisions} divisions"
        );
    }
}

#[test]
fn given_a_finished_autoset_when_the_outcome_is_read_then_it_names_the_final_scales() {
    let (rig, _) = rig_on(Signal::sine(1e3, 1.0));

    let outcome = run(&rig).outcome.unwrap();

    assert_eq!(outcome.text, "Auto-setup done. CH1 200 mV/div, 500 us/div.");
}

#[test]
fn given_a_stopped_scope_when_autoset_is_done_then_capturing_stays_on_and_the_run_intent_is_run() {
    let (rig, _) = rig_on(Signal::sine(1e3, 1.0));

    run(&rig);

    assert_eq!(rig.device.capturing_calls(), [true]);
    assert!(rig.snapshot().instrument.unwrap().running);
}

#[test]
fn given_a_run_when_it_ends_then_the_screen_holds_only_frames_of_the_final_setup() {
    let (rig, _) = rig_on(Signal::sine(1e3, 1.0));

    run(&rig);
    let last = rig.setup_revision();
    rig.wait(|| !rig.frames.events().is_empty());

    assert!(rig.frames.events().iter().all(|f| f.context.setup_revision == last));
    assert!(rig.device.applies().len() > 3, "probes were sent");
}

#[test]
fn given_a_run_in_progress_when_a_probe_frame_arrives_then_nothing_is_published_but_the_next_probe_is_sent() {
    let rig = silent_rig();
    rig.bus.request(StartAutoset).unwrap();
    let first_probe = rig.setup_revision();

    rig.complete_auto(first_probe);
    rig.wait(|| rig.setup_revision() > first_probe);

    assert!(rig.frames.events().is_empty());
}

#[test]
fn given_a_flat_input_when_autoset_runs_then_no_signal_is_flagged_with_1_volt_and_1_ms_per_division() {
    let (rig, _) = rig_on(Signal::flat());

    let status = run(&rig);

    assert_eq!(status.outcome.unwrap().kind, AutosetOutcomeKind::NoSignal);
    assert!(status.no_signal);
    let setup = last_setup(&rig);
    assert_eq!(volts_per_div(&setup), 1.0);
    assert_eq!(seconds_per_div(&setup), 1e-3);
}

#[test]
fn given_no_signal_when_a_later_frame_has_signal_then_the_flag_clears() {
    let (rig, input) = rig_on(Signal::flat());
    run(&rig);
    input.set(Signal::sine(1e3, 1.0));

    let frame = input.frame(&last_setup(&rig));
    rig.complete(rig.setup_revision(), CaptureOutcome::Frame(Arc::new(frame)));

    rig.wait(|| !status(&rig).no_signal);
}

#[test]
fn given_no_signal_when_the_next_autoset_starts_then_the_flag_clears_at_once() {
    let (rig, input) = rig_on(Signal::flat());
    run(&rig);
    input.set(Signal::sine(1e3, 1.0));

    rig.bus.request(StartAutoset).unwrap();

    assert!(!status(&rig).no_signal);
    wait_for_outcome(&rig);
}

#[test]
fn given_two_runs_when_each_ends_then_the_outcomes_carry_different_ids() {
    let (rig, _) = rig_on(Signal::sine(1e3, 1.0));
    let first = run(&rig).outcome.unwrap().id;

    rig.bus.request(StartAutoset).unwrap();
    rig.wait(|| status(&rig).outcome.is_some_and(|o| o.id != first));

    assert!(status(&rig).outcome.unwrap().id > first);
}

#[test]
fn given_a_signal_below_5_hz_when_autoset_runs_then_it_fails_and_the_previous_setup_is_sent_again() {
    let (rig, _) = rig_on(Signal::sine(2.0, 1.0));
    let before = rig.snapshot().instrument.unwrap();
    let original = last_setup(&rig);

    let outcome = run(&rig).outcome.unwrap();

    assert_eq!(outcome.kind, AutosetOutcomeKind::Failed);
    assert_eq!(outcome.text, FAILED);
    assert_eq!(last_setup(&rig), original);
    let after = rig.snapshot().instrument.unwrap();
    assert_eq!(after.channels, before.channels);
    assert_eq!(after.horizontal, before.horizontal);
}

#[test]
fn given_a_stopped_scope_when_autoset_fails_then_capturing_is_switched_off_again() {
    let (rig, _) = rig_on(Signal::sine(2.0, 1.0));

    run(&rig);

    assert_eq!(rig.device.capturing_calls(), [true, false]);
    assert!(!rig.snapshot().instrument.unwrap().running);
}

#[test]
fn given_a_running_scope_when_autoset_fails_then_it_keeps_running_without_new_capturing_calls() {
    let (rig, _) = rig_on(Signal::sine(2.0, 1.0));
    rig.bus.request(SetRunning { running: true }).unwrap();

    run(&rig);

    assert_eq!(rig.device.capturing_calls(), [true]);
    assert!(rig.snapshot().instrument.unwrap().running);
}

#[test]
fn given_a_run_in_progress_when_the_scope_is_unplugged_then_the_run_aborts_with_a_failed_outcome() {
    let rig = silent_rig();
    rig.bus.request(StartAutoset).unwrap();
    assert!(status(&rig).running);

    rig.unplug();

    let status = wait_for_outcome(&rig);
    assert_eq!(status.outcome.unwrap().kind, AutosetOutcomeKind::Failed);
}

#[test]
fn given_a_run_in_progress_when_15_seconds_pass_then_it_fails_and_the_setup_is_restored() {
    let rig = silent_rig();
    let original = last_setup(&rig);
    rig.bus.request(StartAutoset).unwrap();

    rig.clock.advance(Duration::from_secs(15));

    let outcome = wait_for_outcome(&rig).outcome.unwrap();
    assert_eq!(outcome.kind, AutosetOutcomeKind::Failed);
    assert_eq!(last_setup(&rig), original);
}

#[test]
fn given_a_run_in_progress_when_14_seconds_pass_then_the_next_probe_is_still_sent() {
    let rig = silent_rig();
    rig.bus.request(StartAutoset).unwrap();
    let first_probe = rig.setup_revision();

    rig.clock.advance(Duration::from_secs(14));
    rig.complete_auto(first_probe);

    rig.wait(|| rig.setup_revision() > first_probe);
    assert!(status(&rig).running);
}

#[test]
fn given_a_run_in_progress_when_a_setup_edit_arrives_then_it_is_refused_with_the_running_reason() {
    let rig = silent_rig();
    rig.bus.request(StartAutoset).unwrap();

    let message = expect_recoverable(rig.bus.request(SetChannel {
        channel: 1,
        patch: ChannelPatch {
            position_divs: Some(1.0),
            ..ChannelPatch::default()
        },
    }));

    assert_eq!(message, RUNNING);
}

#[test]
fn given_a_run_in_progress_when_run_or_a_second_autoset_is_requested_then_both_are_refused() {
    let rig = silent_rig();
    rig.bus.request(StartAutoset).unwrap();

    assert_eq!(
        expect_recoverable(rig.bus.request(SetRunning { running: true })),
        RUNNING
    );
    assert_eq!(expect_recoverable(rig.bus.request(StartAutoset)), RUNNING);
}

#[test]
fn given_a_run_in_progress_when_the_instrument_is_read_then_the_controls_carry_the_running_reason() {
    let rig = silent_rig();
    rig.bus.request(StartAutoset).unwrap();

    let controls = rig.snapshot().instrument.unwrap().controls;

    assert_eq!(controls.run.reason.as_deref(), Some(RUNNING));
    assert_eq!(controls.autoset.reason.as_deref(), Some(RUNNING));
}

#[test]
fn given_a_run_in_progress_when_the_first_probe_is_sent_then_it_uses_auto_sweep_at_the_1_us_rung() {
    let rig = silent_rig();
    rig.bus.request(StartAutoset).unwrap();

    let setup = last_setup(&rig);

    assert_eq!(setup.trigger.sweep, Sweep::Auto);
    assert_eq!(seconds_per_div(&setup), 1e-6);
}

#[test]
fn given_an_armed_single_when_autoset_starts_then_the_single_ends_first() {
    let rig = silent_rig();
    rig.bus.request(ArmSingle).unwrap();

    rig.bus.request(StartAutoset).unwrap();

    assert!(rig.device.calls().contains(&DeviceCall::Capturing(false)));
    assert!(status(&rig).running);
}

#[test]
fn given_only_channel_2_on_when_autoset_runs_then_the_summary_names_channel_2() {
    let (rig, _) = rig_on(Signal::sine(1e3, 1.0));
    for (channel, enabled) in [(2, true), (1, false)] {
        let patch = patch_enabled(enabled);
        rig.bus.request(SetChannel { channel, patch }).unwrap();
    }

    let outcome = run(&rig).outcome.unwrap();

    assert_eq!(outcome.kind, AutosetOutcomeKind::Done);
    assert!(outcome.text.starts_with("Auto-setup done. CH2 "), "{}", outcome.text);
}
