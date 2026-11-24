//! The real device, acquisition and display modules on the simulator, wired by `compose`.
use std::sync::Arc;
use std::time::Duration;

use sweepscope_acquisition::{
    ChannelPatch, ChannelState, GetInstrument, Phase, SetChannel, SetRunning, SettingTarget, StartAutoset, StepSetting,
};
use sweepscope_calibration::domain::offsets::OffsetFile;
use sweepscope_calibration::{
    CloseZeroCalibration, ContinueZeroChannel, GetZeroCalibration, MeasureZeroChannel, OpenZeroCalibration,
    StartZeroCalibration, ZeroCalibrationState, ZeroStep,
};
use sweepscope_device::{SignalKind, SignalSpec, SimulatorControl};
use sweepscope_display::DisplayFrameReady;
use sweepscope_export::{ExportFormat, ExportFrame, SaveScreenshot};
use sweepscope_kernel::contracts::{ConnectionState, FrameCaptured};
use sweepscope_kernel::testing::{eventually, MemoryStorage, ScriptedPicker};
use sweepscope_kernel::{Bus, Storage, SystemClock};
use sweepscope_recording::OsRecordFiles;

use crate::composition::{compose, simulator_source, unix_time_us, Backend, Wiring};
use crate::ipc::snapshot_with;

const WAIT: Duration = Duration::from_secs(10);

pub(crate) fn simulated_backend() -> (Backend, SimulatorControl) {
    simulated_backend_with(Arc::new(ScriptedPicker::default()))
}

fn simulated_backend_with(picker: Arc<ScriptedPicker>) -> (Backend, SimulatorControl) {
    let (backend, control, _storage) = simulated_backend_on(picker);
    (backend, control)
}

fn simulated_backend_on(picker: Arc<ScriptedPicker>) -> (Backend, SimulatorControl, Arc<MemoryStorage>) {
    let clock = Arc::new(SystemClock);
    let source = simulator_source(clock.clone(), true);
    let control = source.control.clone();
    let storage = Arc::new(MemoryStorage::default());
    let backend = compose(Wiring {
        storage: storage.clone(),
        picker,
        record_files: Arc::new(OsRecordFiles),
        clock,
        hardware: None,
        simulator: Some(source),
        unix_time_us,
    })
    .expect("compose");
    (backend, control, storage)
}

fn wait_until_ready(bus: &Bus) {
    assert!(eventually(WAIT, || {
        snapshot_with(bus).is_ok_and(|s| s.connection.state == ConnectionState::Ready && s.instrument.is_some())
    }));
}

#[test]
fn given_the_composed_app_with_the_simulator_when_a_snapshot_is_requested_then_the_device_is_ready_with_an_instrument()
{
    let (backend, _control) = simulated_backend();

    wait_until_ready(&backend.bus);

    let snapshot = snapshot_with(&backend.bus).expect("snapshot");
    assert!(snapshot.connection.simulator_active);
    assert!(!snapshot.instrument.expect("instrument").running);
}

#[test]
fn given_a_ready_simulator_when_running_then_display_frames_arrive_and_the_status_leaves_stopped() {
    let (backend, _control) = simulated_backend();
    wait_until_ready(&backend.bus);
    let frames = backend.bus.subscribe_latest::<DisplayFrameReady>();

    backend.bus.request(SetRunning { running: true }).expect("run");

    assert!(eventually(WAIT, || frames.try_next().is_some()));
    assert_ne!(
        snapshot_with(&backend.bus).expect("snapshot").status.phase,
        Phase::Stopped
    );
}

#[test]
fn given_a_running_simulator_when_unplugged_then_the_connection_drops_and_running_ends() {
    let (backend, control) = simulated_backend();
    wait_until_ready(&backend.bus);
    backend.bus.request(SetRunning { running: true }).expect("run");

    control.unplug();

    assert!(eventually(WAIT, || {
        snapshot_with(&backend.bus).is_ok_and(|s| {
            s.connection.state == ConnectionState::Disconnected
                && s.status.phase == Phase::Stopped
                && s.instrument.is_some_and(|i| !i.running)
        })
    }));
}

#[test]
fn given_a_running_simulator_when_frames_arrive_then_the_snapshot_carries_measurements_for_the_enabled_channels() {
    let (backend, _control) = simulated_backend();
    wait_until_ready(&backend.bus);

    backend.bus.request(SetRunning { running: true }).expect("run");

    assert!(eventually(WAIT, || {
        snapshot_with(&backend.bus).is_ok_and(|s| !s.measurements.channels.is_empty())
    }));
}

#[test]
fn given_a_frame_and_a_cancelled_dialog_when_exported_then_nothing_is_saved() {
    let picker = Arc::new(ScriptedPicker::new(vec![None], vec![]));
    let (backend, _control) = simulated_backend_with(picker);
    wait_until_ready(&backend.bus);
    backend.bus.request(SetRunning { running: true }).expect("run");

    let saved = eventually(WAIT, || {
        backend
            .bus
            .request(ExportFrame {
                format: ExportFormat::Csv,
            })
            .is_ok_and(|saved| saved.is_none())
    });

    assert!(saved);
}

#[test]
fn given_a_body_that_is_not_a_png_when_saved_as_a_screenshot_then_it_is_refused_before_any_dialog() {
    let (backend, _control) = simulated_backend();

    let error = backend
        .bus
        .request(SaveScreenshot {
            png: b"not a png".to_vec(),
        })
        .expect_err("refused");

    assert_eq!(error.category, sweepscope_kernel::ErrorCategory::Recoverable);
}

const SIMULATOR_UNIT: &str = "simulator";
const CHANNELS: usize = 4;
/// The 2 mV/div noise is 24 codes rms, so a frame mean wanders by a few codes; the zero error is 8.
const MEAN_TOLERANCE_CODES: f64 = 3.0;
const TWO_MILLIVOLT_STEPS_FROM_ONE_VOLT: usize = 8;
const TWO_MILLIVOLT_RANGE_INDEX: usize = 0;
const FIVE_MILLIVOLT_RANGE_INDEX: usize = 1;
/// Simulator zero error of channel 1 on 2 mV/div, 0.66 mV, in codes at 24.15 codes per division.
const CHANNEL_ONE_ZERO_ERROR_CODES: f64 = 0.66e-3 / 2.0e-3 * 24.15;

struct ShortedRig {
    backend: Backend,
    control: SimulatorControl,
    storage: Arc<MemoryStorage>,
}

fn shorted_rig() -> ShortedRig {
    let (backend, control, storage) = simulated_backend_on(Arc::new(ScriptedPicker::default()));
    for channel in 0..CHANNELS {
        control.set_signal(channel, SignalSpec::new(SignalKind::Shorted));
    }
    wait_until_ready(&backend.bus);
    assert!(eventually(WAIT, || zero_state(&backend.bus).availability.enabled));
    ShortedRig {
        backend,
        control,
        storage,
    }
}

fn zero_state(bus: &Bus) -> ZeroCalibrationState {
    bus.request(GetZeroCalibration).expect("calibration state")
}

fn wait_for_step(bus: &Bus, step: ZeroStep) {
    assert!(eventually(WAIT, || {
        zero_state(bus).session.is_some_and(|session| session.step == step)
    }));
}

fn wait_for_channel(bus: &Bus, channel: u8) {
    assert!(eventually(WAIT, || {
        zero_state(bus)
            .session
            .is_some_and(|session| session.step == ZeroStep::Channel && session.channel == Some(channel))
    }));
}

fn measure_channels(bus: &Bus, count: u8) {
    bus.request(OpenZeroCalibration).expect("open");
    bus.request(StartZeroCalibration).expect("start");
    for channel in 1..=count {
        wait_for_channel(bus, channel);
        bus.request(MeasureZeroChannel).expect("measure");
    }
}

fn stored_offsets(storage: &MemoryStorage) -> Option<Vec<f64>> {
    let file = OffsetFile::load(storage as &dyn Storage).expect("offset file");
    file.unit(SIMULATOR_UNIT).map(|unit| unit.volts[0].clone())
}

fn select_two_millivolts_per_div(bus: &Bus) {
    for _ in 0..TWO_MILLIVOLT_STEPS_FROM_ONE_VOLT {
        bus.request(StepSetting {
            target: SettingTarget::Range { channel: 1 },
            direction: -1,
        })
        .expect("step range");
    }
}

fn first_frame_after_run(bus: &Bus) -> FrameCaptured {
    let frames = bus.subscribe_latest::<FrameCaptured>();
    bus.request(SetRunning { running: true }).expect("run");
    let mut frame = None;
    assert!(eventually(WAIT, || {
        frame = frames.try_next();
        frame.is_some()
    }));
    frame.expect("frame")
}

fn frame_at_two_millivolts(bus: &Bus) -> FrameCaptured {
    select_two_millivolts_per_div(bus);
    let frames = bus.subscribe_latest::<FrameCaptured>();
    bus.request(SetRunning { running: true }).expect("run");
    let mut frame = None;
    assert!(eventually(WAIT, || {
        frame = frames
            .try_next()
            .filter(|f| f.frame.channels[0].as_ref().is_some_and(|c| c.volts_per_div == 2.0e-3));
        frame.is_some()
    }));
    frame.expect("frame")
}

fn mean_code(codes: &[u8]) -> f64 {
    codes.iter().map(|&code| f64::from(code)).sum::<f64>() / codes.len() as f64
}

#[test]
fn given_a_shorted_simulator_when_all_four_channels_are_measured_then_the_unit_is_calibrated_and_offsets_are_stored() {
    let rig = shorted_rig();

    measure_channels(&rig.backend.bus, 4);
    wait_for_step(&rig.backend.bus, ZeroStep::Done);

    assert!(zero_state(&rig.backend.bus).unit.expect("unit").calibrated);
    let offsets = stored_offsets(&rig.storage).expect("stored offsets");
    assert!((offsets[0] - 0.66e-3).abs() < 0.15e-3, "stored {}", offsets[0]);
}

#[test]
fn given_stored_offsets_when_frames_are_captured_at_two_millivolts_then_zero_code_is_corrected_and_marked_calibrated() {
    let rig = shorted_rig();
    let bus = &rig.backend.bus;
    measure_channels(bus, 4);
    wait_for_step(bus, ZeroStep::Done);
    bus.request(CloseZeroCalibration).expect("close");

    let captured = frame_at_two_millivolts(bus);

    let channel = captured.frame.channels[0].as_ref().expect("channel 1");
    assert!(captured.context.calibrated[0]);
    let stored = stored_offsets(&rig.storage).expect("stored offsets")[0];
    let applied_codes = stored / 2.0e-3 * captured.frame.counts_per_div;
    assert!(
        (channel.zero_code - 128.0 - applied_codes).abs() < 1e-6,
        "zero_code {}",
        channel.zero_code
    );
    assert!(
        applied_codes > 5.0,
        "a nonzero correction at 2 mV/div, got {applied_codes} codes"
    );
    assert!((mean_code(&channel.codes) - channel.zero_code).abs() < MEAN_TOLERANCE_CODES);
}

#[test]
fn given_no_calibration_when_frames_are_captured_at_two_millivolts_then_zero_code_is_the_scope_value_and_not_calibrated(
) {
    let rig = shorted_rig();

    let captured = frame_at_two_millivolts(&rig.backend.bus);

    let channel = captured.frame.channels[0].as_ref().expect("channel 1");
    assert!(!captured.context.calibrated[0]);
    assert_eq!(channel.zero_code, 128.0);
    assert!((mean_code(&channel.codes) - 128.0 - CHANNEL_ONE_ZERO_ERROR_CODES).abs() < MEAN_TOLERANCE_CODES);
}

#[test]
fn given_three_measured_channels_when_the_dialog_is_cancelled_then_nothing_is_stored_and_frames_stay_uncalibrated() {
    let rig = shorted_rig();
    let bus = &rig.backend.bus;
    measure_channels(bus, 3);
    wait_for_channel(bus, 4);

    bus.request(CloseZeroCalibration).expect("close");

    assert!(stored_offsets(&rig.storage).is_none());
    assert!(!zero_state(bus).unit.expect("unit").calibrated);
    assert!(!first_frame_after_run(bus).context.calibrated[0]);
}

#[test]
fn given_an_auto_setup_in_progress_when_the_dialog_is_opened_then_it_is_refused_and_no_session_starts() {
    let rig = shorted_rig();
    let bus = &rig.backend.bus;
    bus.request(StartAutoset).expect("autoset");

    let refused = bus.request(OpenZeroCalibration);

    assert!(refused.is_err());
    assert!(zero_state(bus).session.is_none());
}

fn open_and_measure_channel_one(bus: &Bus) {
    bus.request(OpenZeroCalibration).expect("open");
    bus.request(StartZeroCalibration).expect("start");
    wait_for_channel(bus, 1);
    bus.request(MeasureZeroChannel).expect("measure");
}

#[test]
fn given_a_zero_error_above_the_limit_when_continue_is_sent_then_the_session_completes_and_the_offset_is_stored_and_applied(
) {
    let rig = shorted_rig();
    let bus = &rig.backend.bus;
    rig.control.set_zero_error_volts(0, FIVE_MILLIVOLT_RANGE_INDEX, 6.0e-3);
    open_and_measure_channel_one(bus);
    wait_for_step(bus, ZeroStep::NotShorted);
    let rejected = zero_state(bus).session.expect("session");
    assert!(rejected.can_continue, "{:?}", rejected);

    bus.request(ContinueZeroChannel).expect("continue");
    for channel in 2..=4 {
        wait_for_channel(bus, channel);
        bus.request(MeasureZeroChannel).expect("measure");
    }
    wait_for_step(bus, ZeroStep::Done);

    let state = zero_state(bus);
    assert!(state.unit.expect("unit").calibrated);
    assert_eq!(state.session.expect("session").overridden, [true, false, false, false]);
    let offsets = stored_offsets(&rig.storage).expect("stored offsets");
    assert!(
        (offsets[FIVE_MILLIVOLT_RANGE_INDEX] - 6.0e-3).abs() < 0.5e-3,
        "stored {}",
        offsets[FIVE_MILLIVOLT_RANGE_INDEX]
    );
    bus.request(CloseZeroCalibration).expect("close");
    let captured = frame_at_two_millivolts(bus);
    assert!(captured.context.calibrated[0]);
}

#[test]
fn given_a_zero_error_below_the_widened_limit_when_measured_then_it_passes_without_continue() {
    let rig = shorted_rig();
    let bus = &rig.backend.bus;
    rig.control.set_zero_error_volts(3, TWO_MILLIVOLT_RANGE_INDEX, 1.45e-3);

    measure_channels(bus, 4);
    wait_for_step(bus, ZeroStep::Done);

    assert_eq!(zero_state(bus).session.expect("session").overridden, [false; 4]);
}

#[test]
fn given_a_clipped_gate_reading_when_continue_is_sent_then_it_is_refused_and_the_session_stays_rejected() {
    let rig = shorted_rig();
    let bus = &rig.backend.bus;
    let mut clipped = SignalSpec::new(SignalKind::Shorted);
    clipped.offset_v = 5.0;
    rig.control.set_signal(0, clipped);
    open_and_measure_channel_one(bus);
    wait_for_step(bus, ZeroStep::NotShorted);

    let result = bus.request(ContinueZeroChannel);

    assert!(!zero_state(bus).session.expect("session").can_continue);
    assert!(result.is_err());
    assert!(zero_state(bus).session.is_some_and(|s| s.step == ZeroStep::NotShorted));
}

#[test]
fn given_a_zero_error_above_the_limit_at_two_millivolts_when_continue_is_sent_then_the_offset_is_stored_and_applied() {
    let rig = shorted_rig();
    let bus = &rig.backend.bus;
    rig.control.set_zero_error_volts(0, TWO_MILLIVOLT_RANGE_INDEX, 3.0e-3);
    open_and_measure_channel_one(bus);
    wait_for_step(bus, ZeroStep::NotShorted);
    let rejected = zero_state(bus).session.expect("session");
    assert!(rejected.can_continue, "{:?}", rejected);

    bus.request(ContinueZeroChannel).expect("continue");
    for channel in 2..=4 {
        wait_for_channel(bus, channel);
        bus.request(MeasureZeroChannel).expect("measure");
    }
    wait_for_step(bus, ZeroStep::Done);

    let offsets = stored_offsets(&rig.storage).expect("stored offsets");
    assert!(
        (offsets[TWO_MILLIVOLT_RANGE_INDEX] - 3.0e-3).abs() < 0.5e-3,
        "stored {}",
        offsets[TWO_MILLIVOLT_RANGE_INDEX]
    );
    assert_eq!(
        zero_state(bus).session.expect("session").overridden,
        [true, false, false, false]
    );
}

fn channel_one(bus: &Bus) -> ChannelState {
    let snapshot = bus.request(GetInstrument).expect("instrument");
    snapshot.instrument.expect("instrument slice").channels.remove(0)
}

fn range_label(channel: &ChannelState) -> String {
    format!("{} {}", channel.range.label.text, channel.range.label.unit)
}

fn assert_tooltip_matches_instrument(bus: &Bus, probe_slot: usize) {
    let channel = channel_one(bus);
    let tooltip = zero_state(bus).unit.expect("unit").tooltip;
    let cell = &tooltip.channels[0].cells[usize::from(channel.range.index)][probe_slot];
    assert!(
        cell.ends_with(&format!(" at {}", range_label(&channel))),
        "range index {}: cell {cell:?}, instrument {}",
        channel.range.index,
        range_label(&channel)
    );
}

fn walk_every_range(bus: &Bus, probe_slot: usize) {
    for _ in 0..12 {
        assert_tooltip_matches_instrument(bus, probe_slot);
        bus.request(StepSetting {
            target: SettingTarget::Range { channel: 1 },
            direction: -1,
        })
        .expect("step range");
    }
    for _ in 0..12 {
        assert_tooltip_matches_instrument(bus, probe_slot);
        bus.request(StepSetting {
            target: SettingTarget::Range { channel: 1 },
            direction: 1,
        })
        .expect("step range");
    }
}

#[test]
fn given_every_range_of_channel_one_when_the_probe_is_1x_or_10x_then_the_instrument_index_selects_the_tooltip_cell_with_the_shown_range(
) {
    let rig = shorted_rig();
    let bus = &rig.backend.bus;
    assert!(eventually(WAIT, || bus
        .request(GetInstrument)
        .is_ok_and(|snapshot| snapshot.instrument.is_some())));

    walk_every_range(bus, 0);
    bus.request(SetChannel {
        channel: 1,
        patch: ChannelPatch {
            probe: Some(10.0),
            ..ChannelPatch::default()
        },
    })
    .expect("probe");
    walk_every_range(bus, 1);
}
