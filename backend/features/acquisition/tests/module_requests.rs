#[path = "domain_support/mod.rs"]
mod domain_support;
mod module_support;

use domain_support::{patch_enabled, range_index};
use module_support::{expect_recoverable, ready_rig, rig, DeviceCall};
use sweepscope_acquisition::{
    ChannelPatch, ForceTriggerNow, ResetSetting, SetChannel, SetRunning, SetTrigger, SetTriggerLevel,
    SetTriggerPosition, SettingTarget, StepSetting, TriggerPatch, COMMANDS,
};
use sweepscope_kernel::contracts::{Slope, Sweep};

fn set_running(rig: &module_support::Rig, running: bool) {
    rig.bus.request(SetRunning { running }).unwrap();
}

fn channel_patch(enabled: bool) -> SetChannel {
    SetChannel {
        channel: 2,
        patch: patch_enabled(enabled),
    }
}

#[test]
fn given_no_device_when_run_is_requested_then_it_is_rejected_with_the_reason_and_nothing_is_sent() {
    let rig = rig();

    let reason = expect_recoverable(rig.bus.request(SetRunning { running: true }));

    assert_eq!(reason, "No scope connected");
    assert_eq!(rig.device.calls(), vec![]);
}

#[test]
fn given_no_device_when_the_state_is_asked_then_there_is_no_instrument_and_the_status_is_stopped() {
    let rig = rig();

    let snapshot = rig.snapshot();

    assert!(snapshot.instrument.is_none());
    assert_eq!(snapshot.status.phase, sweepscope_acquisition::Phase::Stopped);
    assert!(snapshot.facts.sample_rate.is_none());
}

#[test]
fn given_a_device_that_just_became_ready_when_the_module_restores_then_the_default_setup_is_applied_once_stopped() {
    let rig = ready_rig();

    let applies = rig.device.applies();

    assert_eq!(applies.len(), 1);
    assert!(applies[0].1.channels[0].enabled);
    assert!(rig.device.capturing_calls().is_empty());
    assert!(!rig.snapshot().instrument.unwrap().running);
}

#[test]
fn given_a_ready_device_when_run_is_requested_then_capturing_starts_and_the_slice_says_running() {
    let rig = ready_rig();

    set_running(&rig, true);

    assert_eq!(rig.device.capturing_calls(), vec![true]);
    assert!(rig.snapshot().instrument.unwrap().running);
}

#[test]
fn given_a_running_device_when_stop_is_requested_then_capturing_stops_and_the_status_is_stopped() {
    let rig = ready_rig();
    set_running(&rig, true);

    set_running(&rig, false);

    assert_eq!(rig.device.capturing_calls(), vec![true, false]);
    assert_eq!(rig.snapshot().status.phase, sweepscope_acquisition::Phase::Stopped);
}

#[test]
fn given_a_running_device_when_run_is_requested_again_then_nothing_is_sent_or_published_twice() {
    let rig = ready_rig();
    set_running(&rig, true);
    let published = rig.instrument.events().len();

    set_running(&rig, true);

    assert_eq!(rig.device.capturing_calls(), vec![true]);
    assert_eq!(rig.instrument.events().len(), published);
}

#[test]
fn given_a_ready_device_when_a_setting_changes_then_exactly_one_apply_setup_with_a_newer_revision_is_sent() {
    let rig = ready_rig();
    let before = rig.setup_revision();

    rig.bus.request(channel_patch(true)).unwrap();

    let applies = rig.device.applies();
    assert_eq!(applies.len(), 2);
    assert!(applies[1].0 > before);
    assert!(applies[1].1.channels[1].enabled);
}

#[test]
fn given_a_change_that_is_not_part_of_the_device_setup_when_it_is_made_then_the_state_is_published_but_no_setup_is_sent(
) {
    let rig = ready_rig();
    let applied = rig.device.applies().len();
    let published = rig.instrument.events().len();

    let patch = ChannelPatch {
        probe: Some(10.0),
        invert: Some(true),
        ..ChannelPatch::default()
    };
    rig.bus.request(SetChannel { channel: 1, patch }).unwrap();

    assert_eq!(rig.device.applies().len(), applied);
    assert_eq!(rig.instrument.events().len(), published + 1);
}

#[test]
fn given_an_edit_that_changes_nothing_when_it_is_made_then_no_event_and_no_setup_are_sent() {
    let rig = ready_rig();
    let applied = rig.device.applies().len();
    let published = rig.instrument.events().len();

    rig.bus.request(channel_patch(false)).unwrap();

    assert_eq!(rig.device.applies().len(), applied);
    assert_eq!(rig.instrument.events().len(), published);
}

#[test]
fn given_the_last_enabled_channel_when_it_is_disabled_then_the_edit_is_rejected_and_nothing_changes() {
    let rig = ready_rig();
    let applied = rig.device.applies().len();
    let published = rig.instrument.events().len();

    let reason = expect_recoverable(rig.bus.request(SetChannel {
        channel: 1,
        patch: patch_enabled(false),
    }));

    assert_eq!(reason, "At least one channel must stay on");
    assert_eq!(rig.device.applies().len(), applied);
    assert_eq!(rig.instrument.events().len(), published);
}

#[test]
fn given_several_changes_when_each_is_made_then_the_instrument_revision_grows_by_one_per_change_and_the_snapshot_equals_the_last_event(
) {
    let rig = ready_rig();
    rig.bus.request(channel_patch(true)).unwrap();
    rig.bus
        .request(StepSetting {
            target: SettingTarget::TimeBase,
            direction: 1,
        })
        .unwrap();
    rig.bus
        .request(SetTrigger {
            patch: TriggerPatch {
                slope: Some(Slope::Falling),
                ..TriggerPatch::default()
            },
        })
        .unwrap();

    let events = rig.instrument.events();

    let revisions: Vec<u64> = events.iter().map(|e| e.0.revision).collect();
    assert_eq!(revisions, (1..=revisions.len() as u64).collect::<Vec<_>>());
    assert_eq!(rig.snapshot().instrument.as_ref(), events.last().map(|e| &e.0));
}

#[test]
fn given_a_ready_device_when_each_edit_kind_is_requested_then_all_reach_the_domain_and_the_device() {
    let rig = ready_rig();
    let caps = domain_support::capabilities();
    let one_volt = range_index(&caps, 1.0);

    rig.bus
        .request(StepSetting {
            target: SettingTarget::Range { channel: 1 },
            direction: -1,
        })
        .unwrap();
    rig.bus
        .request(ResetSetting {
            target: SettingTarget::Range { channel: 1 },
        })
        .unwrap();
    rig.bus.request(SetTriggerPosition { fraction: 0.25 }).unwrap();
    rig.bus.request(SetTriggerLevel { divisions: 1.0 }).unwrap();
    rig.bus
        .request(SetTrigger {
            patch: TriggerPatch {
                sweep: Some(Sweep::Normal),
                ..TriggerPatch::default()
            },
        })
        .unwrap();

    let last = rig.device.applies().pop().unwrap().1;
    assert_eq!(last.channels[0].range_index, one_volt);
    assert_eq!(last.trigger.sweep, Sweep::Normal);
    assert!(last.trigger.level_volts > 0.0);
    assert!(last.trigger.horizontal_position_percent < 50.0);
}

#[test]
fn given_a_value_that_is_not_a_number_when_it_is_requested_then_the_edit_is_rejected_with_a_reason() {
    let rig = ready_rig();

    let reason = expect_recoverable(rig.bus.request(SetTriggerLevel { divisions: f64::NAN }));

    assert_eq!(reason, "The value must be a number");
}

#[test]
fn given_a_running_device_when_force_trigger_is_requested_then_the_device_is_asked_once() {
    let rig = ready_rig();
    set_running(&rig, true);

    rig.bus.request(ForceTriggerNow).unwrap();

    assert_eq!(rig.device.forces(), 1);
}

#[test]
fn given_a_stopped_device_when_force_trigger_is_requested_then_it_is_rejected_with_the_availability_reason() {
    let rig = ready_rig();

    let reason = expect_recoverable(rig.bus.request(ForceTriggerNow));

    assert!(!reason.is_empty());
    assert_eq!(rig.device.forces(), 0);
}

#[test]
fn given_no_device_when_force_trigger_is_requested_then_it_is_rejected_with_the_connection_reason() {
    let rig = rig();

    let reason = expect_recoverable(rig.bus.request(ForceTriggerNow));

    assert_eq!(reason, "No scope connected");
}

#[test]
fn given_the_module_when_its_commands_are_listed_then_they_are_the_acquisition_commands_of_the_spec() {
    let names: Vec<&str> = COMMANDS.iter().map(|c| c.name).collect();

    assert_eq!(
        names,
        [
            "set_running",
            "force_trigger",
            "set_channel",
            "step_setting",
            "reset_setting",
            "set_trigger_position",
            "set_trigger_level",
            "set_trigger",
        ]
    );
}

#[test]
fn given_an_acquisition_module_when_a_second_one_starts_on_the_same_bus_then_it_is_refused() {
    let rig = rig();

    let second = sweepscope_acquisition::AcquisitionModule::new(sweepscope_acquisition::AcquisitionDeps {
        bus: rig.bus.clone(),
        clock: std::sync::Arc::new(rig.clock.clone()),
    });

    assert!(second.is_err());
}

#[test]
fn given_a_module_with_a_running_thread_when_it_is_dropped_then_it_returns_promptly() {
    let rig = ready_rig();
    let started = std::time::Instant::now();

    drop(rig.module);

    assert!(started.elapsed() < std::time::Duration::from_secs(1));
    assert!(matches!(rig.device.calls().first(), Some(DeviceCall::Apply(..))));
}
