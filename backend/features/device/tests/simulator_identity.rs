mod common;

use common::*;
use sweepscope_device::{CalibrationStatus, ScopeDevice};

#[test]
fn given_a_simulated_device_when_identity_is_read_then_it_is_simulated_with_firmware_fpga_and_calibration() {
    let rig = rig();

    let identity = rig.device.identity();

    assert!(identity.model.contains("simulated"));
    assert!(identity.simulated);
    assert!(identity.firmware.is_some());
    assert!(identity.fpga_version.is_some());
    assert_eq!(identity.calibration, CalibrationStatus::Valid);
}

#[test]
fn given_one_two_and_four_enabled_channels_when_the_sample_interval_is_asked_at_2_ns_then_it_is_1_2_and_4_ns() {
    let rig = rig();
    let caps = rig.device.capabilities();

    assert_eq!(caps.sample_interval_s(1, TB_2NS), 1e-9);
    assert_eq!(caps.sample_interval_s(2, TB_2NS), 2e-9);
    assert_eq!(caps.sample_interval_s(4, TB_2NS), 4e-9);
}

#[test]
fn given_the_200_us_per_div_time_base_when_the_sample_interval_is_asked_then_it_is_the_time_base_over_250() {
    let rig = rig();

    let interval = rig.device.capabilities().sample_interval_s(4, TB_200US);

    assert!((interval - 800e-9).abs() < 1e-15, "{interval}");
}

#[test]
fn given_the_capabilities_when_read_then_the_time_base_constants_of_the_tests_point_at_the_right_steps() {
    let rig = rig();
    let bases = &rig.device.capabilities().time_bases;

    assert_eq!(bases[usize::from(TB_200US)].seconds_per_div, 200e-6);
    assert_eq!(bases[usize::from(TB_200MS)].seconds_per_div, 0.2);
    assert_eq!(bases[usize::from(TB_500MS)].seconds_per_div, 0.5);
}
