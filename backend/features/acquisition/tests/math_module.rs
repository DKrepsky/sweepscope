#[path = "domain_support/mod.rs"]
mod domain_support;
mod module_support;

use std::sync::Arc;

use module_support::{expect_recoverable, ready_rig, Rig};
use sweepscope_acquisition::{
    ChannelPatch, MathOperation, MathPatch, MathStatus, SetChannel, SetMath, SetRunning, SettingTarget, StepSetting,
};
use sweepscope_kernel::contracts::{CaptureOutcome, ChannelData, DerivedUnit, Frame, TriggerOutcome};

fn two_channel_frame() -> Arc<Frame> {
    let channel = |codes: Vec<u8>| {
        Some(ChannelData {
            codes,
            volts_per_div: 1.0,
            zero_code: 128.0,
        })
    };
    Arc::new(Frame {
        channels: [channel(vec![153, 128]), channel(vec![128, 153]), None, None],
        counts_per_div: 25.0,
        sample_interval_s: 1e-6,
        trigger: TriggerOutcome::Auto,
    })
}

fn running_with_math() -> Rig {
    let rig = ready_rig();
    rig.bus
        .request(SetChannel {
            channel: 2,
            patch: ChannelPatch {
                enabled: Some(true),
                ..ChannelPatch::default()
            },
        })
        .unwrap();
    rig.bus.request(SetRunning { running: true }).unwrap();
    rig.bus
        .request(SetMath {
            patch: MathPatch {
                enabled: Some(true),
                ..MathPatch::default()
            },
        })
        .unwrap();
    rig
}

fn deliver_frame(rig: &Rig) {
    let seen = rig.frames.events().len();
    rig.complete(rig.setup_revision(), CaptureOutcome::Frame(two_channel_frame()));
    rig.wait(|| rig.frames.events().len() > seen);
}

#[test]
fn given_math_on_with_two_channels_when_a_frame_is_captured_then_it_carries_the_derived_trace() {
    let rig = running_with_math();

    deliver_frame(&rig);

    let derived = rig.frames.events()[0].derived.clone().expect("a derived trace");
    assert_eq!(derived.unit, DerivedUnit::Volts);
    assert!((derived.volts[0] - 1.0).abs() < 1e-9 && (derived.volts[1] + 1.0).abs() < 1e-9);
}

#[test]
fn given_math_off_when_a_frame_is_captured_then_it_carries_no_derived_trace() {
    let rig = running_with_math();
    rig.bus
        .request(SetMath {
            patch: MathPatch {
                enabled: Some(false),
                ..MathPatch::default()
            },
        })
        .unwrap();

    deliver_frame(&rig);

    assert!(rig.frames.events()[0].derived.is_none());
}

#[test]
fn given_ch2_switched_off_when_a_frame_is_captured_then_there_is_no_trace_and_the_state_says_why() {
    let rig = running_with_math();
    rig.bus
        .request(SetChannel {
            channel: 2,
            patch: ChannelPatch {
                enabled: Some(false),
                ..ChannelPatch::default()
            },
        })
        .unwrap();

    deliver_frame(&rig);

    assert!(rig.frames.events()[0].derived.is_none());
    assert_eq!(
        rig.snapshot().instrument.unwrap().math.status,
        MathStatus::Unavailable {
            reason: "CH2 is off".to_owned()
        }
    );
}

#[test]
fn given_math_enabled_when_the_state_is_published_then_the_instrument_event_carries_it_without_a_new_setup() {
    let rig = running_with_math();
    let applies = rig.device.applies().len();

    rig.bus
        .request(SetMath {
            patch: MathPatch {
                operation: Some(MathOperation::Add),
                ..MathPatch::default()
            },
        })
        .unwrap();

    let last = rig.instrument.events().last().unwrap().0.clone();
    assert_eq!(last.math.operation, MathOperation::Add);
    assert_eq!(rig.device.applies().len(), applies);
}

#[test]
fn given_a_math_scale_step_request_when_it_is_handled_then_the_scale_changes() {
    let rig = running_with_math();
    let before = rig.snapshot().instrument.unwrap().math.scale.index;

    rig.bus
        .request(StepSetting {
            target: SettingTarget::MathScale,
            direction: 1,
        })
        .unwrap();

    assert_eq!(rig.snapshot().instrument.unwrap().math.scale.index, before + 1);
}

#[test]
fn given_roll_mode_when_math_is_enabled_then_the_request_is_rejected_with_the_roll_reason() {
    let rig = ready_rig();
    rig.step_time_base_until_roll(true);

    let message = expect_recoverable(rig.bus.request(SetMath {
        patch: MathPatch {
            enabled: Some(true),
            ..MathPatch::default()
        },
    }));

    assert!(message.contains("not available in roll mode"));
}

#[test]
fn given_a_frame_held_while_stopped_when_the_operation_changes_then_the_frame_is_published_again_with_the_new_trace() {
    let rig = running_with_math();
    deliver_frame(&rig);
    rig.bus.request(SetRunning { running: false }).unwrap();
    let seen = rig.frames.events().len();

    rig.bus
        .request(SetMath {
            patch: MathPatch {
                operation: Some(MathOperation::Multiply),
                ..MathPatch::default()
            },
        })
        .unwrap();

    rig.wait(|| rig.frames.events().len() > seen);
    let derived = rig
        .frames
        .events()
        .last()
        .unwrap()
        .derived
        .clone()
        .expect("a derived trace");
    assert_eq!(derived.unit, DerivedUnit::VoltsSquared);
    assert_eq!(derived.volts, vec![0.0, 0.0]);
}

#[test]
fn given_a_frame_held_while_stopped_when_the_state_changes_without_touching_math_then_nothing_is_published_again() {
    let rig = running_with_math();
    deliver_frame(&rig);
    rig.bus.request(SetRunning { running: false }).unwrap();
    let seen = rig.frames.events().len();

    rig.bus
        .request(SetMath {
            patch: MathPatch {
                enabled: Some(true),
                ..MathPatch::default()
            },
        })
        .unwrap();

    assert_eq!(rig.frames.events().len(), seen);
}
