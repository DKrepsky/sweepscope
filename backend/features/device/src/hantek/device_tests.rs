//! `HantekDevice` against the driver's scripted `MockTransport`: no USB involved.

use std::time::Duration;

use hantek6xx4_lib::protocol::{CAL_FLAG, CAL_WORDS};
use hantek6xx4_lib::transport::Call;
use hantek6xx4_lib::{CaptureOptions, MockTransport, Scope, Sweep as DriverSweep};

use super::convert::capture_frame;
use super::HantekDevice;
use crate::{
    AcquisitionSetup, CalibrationStatus, ChannelSetup, Coupling, DeviceError, ScopeDevice, Slope, Sweep, TriggerKind,
    TriggerOutcome, TriggerSetup,
};

const TIMEOUT: Duration = Duration::from_secs(2);
const END_ADDRESS: [u8; 2] = [0xf0, 0x4f];

fn answer_control_in(request: u8, value: u16, len: usize, valid_calibration: bool) -> Vec<u8> {
    let mut bytes = vec![0u8; len];
    match (request, value) {
        (0xB2, _) => bytes[0] = 1,
        (0xE3, 0x15F0) => bytes.copy_from_slice(&[0x20, 0x16, 0x05, 0x31]),
        (0xA2, 0x1580) => {
            bytes.fill(0xff);
            bytes[..3].copy_from_slice(b"DSO");
            bytes[6..10].copy_from_slice(b"6000");
            bytes[14..19].copy_from_slice(b"P1.06");
        }
        (0xA2, 0x1B00) => {
            let mut words = vec![0x400u16; 289];
            words[288] = 0xFACF;
            bytes = words.iter().flat_map(|word| word.to_le_bytes()).collect();
        }
        (0xA2, 0x1600) => {
            let mut words = vec![0u16; CAL_WORDS];
            if valid_calibration {
                words[CAL_WORDS - 1] = CAL_FLAG;
            }
            bytes = words.iter().flat_map(|word| word.to_le_bytes()).collect();
        }
        _ => {}
    }
    bytes
}

fn mock(valid_calibration: bool, triggered: bool) -> MockTransport {
    let mut transport = MockTransport::new();
    transport.on_control_in(move |request, value, len| answer_control_in(request, value, len, valid_calibration));
    transport.push_bulk_in([0x01u8, 0xd0].repeat(256));
    transport.on_bulk_in(move |calls, len| {
        let last_command = calls.iter().rev().find_map(|call| match call {
            Call::BulkOut(data) => Some(data.clone()),
            _ => None,
        });
        let mut reply = vec![50u8; len];
        match last_command.as_deref() {
            Some([0x06, 0x00]) => reply[..2].copy_from_slice(&[if triggered { 0x03 } else { 0x02 }, 0xc4]),
            Some([0x0d, 0x00]) => {
                reply[..2].fill(0);
                reply[2..4].copy_from_slice(&END_ADDRESS);
            }
            Some([0x05, 0x00, ..]) => {}
            _ => reply.fill(0),
        }
        reply
    });
    transport
}

fn device() -> HantekDevice<MockTransport> {
    HantekDevice::new(mock(true, true)).unwrap()
}

fn setup(sweep: Sweep) -> AcquisitionSetup {
    let channel = |enabled| ChannelSetup {
        enabled,
        range_index: 8,
        position_divs: 0.0,
        coupling: Coupling::Dc,
        bandwidth_limit: false,
    };
    AcquisitionSetup {
        channels: [channel(true), channel(false), channel(true), channel(true)],
        time_base_index: 15,
        trigger: TriggerSetup {
            source: 0,
            level_volts: 0.0,
            slope: Slope::Rising,
            sweep,
            horizontal_position_percent: 50.0,
            kind: TriggerKind::Edge,
        },
    }
}

fn bulk_writes(device: &HantekDevice<MockTransport>) -> Vec<Vec<u8>> {
    device.scope.transport().bulk_writes()
}

fn start_commands(device: &HantekDevice<MockTransport>) -> Vec<Vec<u8>> {
    bulk_writes(device)
        .into_iter()
        .filter(|write| write.len() == 4 && write[..2] == [0x03, 0x00])
        .collect()
}

fn configure_count(device: &HantekDevice<MockTransport>) -> usize {
    bulk_writes(device).iter().filter(|write| write[0] == 0x10).count()
}

#[test]
fn given_a_scripted_scope_when_opened_then_the_identity_is_labelled_with_what_the_device_reports() {
    let device = device();

    let identity = device.identity();

    assert!(
        identity.model.contains("6000") && identity.model.contains("P1.06"),
        "{}",
        identity.model
    );
    assert_eq!(identity.firmware.as_deref(), Some("driver version 20160531"));
    assert_eq!(identity.fpga_version.as_deref(), Some("0xD001"));
    assert_eq!(identity.calibration, CalibrationStatus::Valid);
    assert!(!identity.simulated);
}

#[test]
fn given_a_scope_when_opened_then_the_capabilities_are_the_shared_tables() {
    assert_eq!(device().capabilities(), &crate::profile::capabilities());
}

#[test]
fn given_a_scope_with_an_invalid_calibration_table_when_opened_then_calibration_invalid() {
    let result = HantekDevice::new(mock(false, true));

    assert!(matches!(result, Err(DeviceError::CalibrationInvalid)));
}

#[test]
fn given_a_configured_scope_when_captured_then_the_frame_equals_the_converted_driver_frame() {
    let mut device = device();
    device.configure(&setup(Sweep::Normal)).unwrap();
    let mut reference = Scope::new(mock(true, true)).unwrap();
    reference
        .configure(&super::convert::config_from_setup(&setup(Sweep::Normal)).unwrap())
        .unwrap();
    let expected = capture_frame(&reference.capture(CaptureOptions { timeout: TIMEOUT }).unwrap());

    let frame = device.capture(TIMEOUT).unwrap();

    assert_eq!(frame, expected);
    assert_eq!(frame.trigger, TriggerOutcome::Triggered { sample_index: 2048 });
    assert_eq!(frame.channels[0].as_ref().unwrap().codes.len(), 4096);
    assert!(frame.channels[1].is_none());
}

#[test]
fn given_an_untriggered_capture_when_read_then_the_frame_is_an_auto_frame() {
    let mut device = HantekDevice::new(mock(true, false)).unwrap();
    device.configure(&setup(Sweep::Auto)).unwrap();

    let frame = device.capture(TIMEOUT).unwrap();

    assert_eq!(frame.trigger, TriggerOutcome::Auto);
}

#[test]
fn given_a_scope_that_was_never_configured_when_captured_then_the_setting_is_invalid() {
    let result = device().capture(TIMEOUT);

    assert!(matches!(result, Err(DeviceError::InvalidSetting(_))));
}

#[test]
fn given_an_invalid_setup_when_configured_then_it_is_rejected_before_the_device_is_touched() {
    let mut device = device();
    let before = bulk_writes(&device).len();
    let mut bad = setup(Sweep::Auto);
    bad.time_base_index = 99;

    let result = device.configure(&bad);

    assert!(matches!(result, Err(DeviceError::InvalidSetting(_))));
    assert_eq!(bulk_writes(&device).len(), before);
}

#[test]
fn given_normal_sweep_when_forced_then_an_auto_configure_precedes_the_capture() {
    let mut device = device();
    device.configure(&setup(Sweep::Normal)).unwrap();

    device.force_capture(TIMEOUT).unwrap();

    assert_eq!(device.scope.config().trigger.sweep, DriverSweep::Auto);
    assert_eq!(start_commands(&device), vec![vec![0x03, 0x00, 0x01, 0x00]]);
    assert_eq!(configure_count(&device), 2);
}

#[test]
fn given_a_forced_capture_when_the_next_normal_capture_runs_then_the_requested_sweep_is_applied_first() {
    let mut device = device();
    device.configure(&setup(Sweep::Normal)).unwrap();
    device.force_capture(TIMEOUT).unwrap();

    device.capture(TIMEOUT).unwrap();

    assert_eq!(device.scope.config().trigger.sweep, DriverSweep::Normal);
    assert_eq!(
        start_commands(&device),
        vec![vec![0x03, 0x00, 0x01, 0x00], vec![0x03, 0x00, 0x00, 0x00]]
    );
    assert_eq!(configure_count(&device), 3);
}

#[test]
fn given_a_restored_setup_when_captured_again_then_it_is_not_configured_again() {
    let mut device = device();
    device.configure(&setup(Sweep::Normal)).unwrap();
    device.force_capture(TIMEOUT).unwrap();
    device.capture(TIMEOUT).unwrap();

    device.capture(TIMEOUT).unwrap();

    assert_eq!(configure_count(&device), 3);
}

#[test]
fn given_auto_sweep_when_forced_then_no_extra_configure_happens() {
    let mut device = device();
    device.configure(&setup(Sweep::Auto)).unwrap();

    device.force_capture(TIMEOUT).unwrap();
    device.capture(TIMEOUT).unwrap();

    assert_eq!(configure_count(&device), 1);
}

#[test]
fn given_a_forced_capture_when_a_new_setup_is_configured_then_the_next_capture_does_not_restore_the_old_one() {
    let mut device = device();
    device.configure(&setup(Sweep::Normal)).unwrap();
    device.force_capture(TIMEOUT).unwrap();
    device.configure(&setup(Sweep::Normal)).unwrap();

    device.capture(TIMEOUT).unwrap();

    assert_eq!(configure_count(&device), 3);
}

#[test]
fn given_a_capturing_scope_when_stopped_then_the_stop_command_is_sent() {
    let mut device = device();

    device.stop().unwrap();

    assert_eq!(bulk_writes(&device).last(), Some(&vec![0x03, 0x00, 0x04, 0x00]));
}

#[test]
fn given_a_started_roll_when_polled_then_the_chunk_has_no_trigger() {
    let mut device = device();
    device.configure(&setup(Sweep::Auto)).unwrap();
    device.start_roll().unwrap();

    let chunk = device.poll_roll().unwrap();

    assert_eq!(chunk.trigger, TriggerOutcome::None);
}

#[test]
fn given_no_started_roll_when_polled_then_the_setting_is_invalid() {
    let mut device = device();
    device.configure(&setup(Sweep::Auto)).unwrap();

    assert!(matches!(device.poll_roll(), Err(DeviceError::InvalidSetting(_))));
}
