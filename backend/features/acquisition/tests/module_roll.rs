#[path = "domain_support/mod.rs"]
mod domain_support;
mod module_support;

use module_support::{ready_rig, Rig};
use sweepscope_acquisition::{ChannelPatch, SetChannel, SetRunning, SettingTarget, StepSetting};
use sweepscope_kernel::contracts::{RollCaptured, RollItem};

fn start_rolling() -> Rig {
    let rig = ready_rig();
    rig.step_time_base_until_roll(true);
    rig.bus.request(SetRunning { running: true }).unwrap();
    rig.wait(|| !rig.roll_items().is_empty());
    rig
}

fn kinds(items: &[RollCaptured]) -> Vec<String> {
    items
        .iter()
        .map(|captured| match &captured.item {
            RollItem::Opened => "opened".to_owned(),
            RollItem::Samples { first_index, .. } => format!("samples {first_index}"),
            RollItem::Gap { after, missed } => format!("gap {after} {missed}"),
            RollItem::Closed { reason } => format!("closed {reason:?}"),
        })
        .collect()
}

fn sessions(items: &[RollCaptured]) -> Vec<u32> {
    items.iter().map(|captured| captured.session).collect()
}

#[test]
fn given_a_roll_time_base_when_run_then_a_session_opens_with_the_roll_context() {
    let rig = start_rolling();

    let items = rig.roll_items();

    assert_eq!(kinds(&items), ["opened"]);
    assert_eq!(items[0].session, 1);
    assert_eq!(items[0].context.session, 1);
    assert_eq!(items[0].context.seconds_per_div, 0.5);
}

#[test]
fn given_a_roll_time_base_when_stopped_without_running_then_no_session_opens() {
    let rig = ready_rig();

    rig.step_time_base_until_roll(true);
    rig.unplug();

    assert!(rig.roll_items().is_empty());
}

#[test]
fn given_a_frame_time_base_when_run_then_no_roll_item_is_published() {
    let rig = ready_rig();

    rig.bus.request(SetRunning { running: true }).unwrap();
    rig.unplug();

    assert!(rig.roll_items().iter().all(|item| item.item != RollItem::Opened));
}

#[test]
fn given_a_polled_chunk_of_the_current_setup_when_rolling_then_it_is_published_in_the_session() {
    let rig = start_rolling();

    rig.poll_samples(rig.setup_revision(), 0);
    rig.wait(|| rig.roll_items().len() == 2);

    let items = rig.roll_items();
    assert_eq!(kinds(&items), ["opened", "samples 0"]);
    assert_eq!(sessions(&items), [1, 1]);
}

#[test]
fn given_a_stale_polled_revision_when_rolling_then_nothing_is_published_for_it() {
    let rig = start_rolling();
    let revision = rig.setup_revision();

    rig.poll_samples(revision - 1, 0);
    rig.poll_samples(revision, 7);
    rig.wait(|| rig.roll_items().len() == 2);

    assert_eq!(kinds(&rig.roll_items()), ["opened", "samples 7"]);
}

#[test]
fn given_roll_running_when_the_user_stops_then_the_session_closes_as_stopped() {
    let rig = start_rolling();

    rig.bus.request(SetRunning { running: false }).unwrap();
    rig.wait(|| rig.roll_items().len() == 2);

    assert_eq!(kinds(&rig.roll_items()), ["opened", "closed Stopped"]);
}

#[test]
fn given_roll_stopped_when_a_chunk_is_polled_then_it_is_not_published() {
    let rig = start_rolling();
    let revision = rig.setup_revision();
    rig.bus.request(SetRunning { running: false }).unwrap();

    rig.poll_samples(revision, 0);
    rig.unplug();

    assert_eq!(kinds(&rig.roll_items()), ["opened", "closed Stopped"]);
}

#[test]
fn given_roll_running_when_the_time_base_steps_to_a_frame_step_then_the_session_closes_as_left_roll() {
    let rig = start_rolling();

    rig.step_time_base_until_roll(false);
    rig.wait(|| rig.roll_items().len() == 2);

    assert_eq!(kinds(&rig.roll_items()), ["opened", "closed LeftRoll"]);
}

#[test]
fn given_a_frame_time_base_running_when_it_steps_into_roll_then_a_session_opens() {
    let rig = ready_rig();
    rig.bus.request(SetRunning { running: true }).unwrap();

    rig.step_time_base_until_roll(true);
    rig.wait(|| !rig.roll_items().is_empty());

    assert_eq!(kinds(&rig.roll_items()), ["opened"]);
}

#[test]
fn given_roll_running_when_a_vertical_setting_changes_then_the_session_closes_and_a_new_one_opens() {
    let rig = start_rolling();

    rig.bus
        .request(StepSetting {
            target: SettingTarget::Range { channel: 1 },
            direction: 1,
        })
        .unwrap();
    rig.wait(|| rig.roll_items().len() == 3);

    let items = rig.roll_items();
    assert_eq!(kinds(&items), ["opened", "closed SetupChanged", "opened"]);
    assert_eq!(sessions(&items), [1, 1, 2]);
}

#[test]
fn given_roll_running_when_a_channel_is_inverted_then_a_new_session_opens_with_the_flag_in_its_context() {
    let rig = start_rolling();

    rig.bus
        .request(SetChannel {
            channel: 1,
            patch: ChannelPatch {
                invert: Some(true),
                ..ChannelPatch::default()
            },
        })
        .unwrap();
    rig.wait(|| rig.roll_items().len() == 3);

    let items = rig.roll_items();
    assert_eq!(kinds(&items), ["opened", "closed SetupChanged", "opened"]);
    assert!(items[2].context.invert[0]);
}

#[test]
fn given_a_new_session_when_a_chunk_of_the_old_revision_arrives_then_it_is_dropped() {
    let rig = start_rolling();
    let old = rig.setup_revision();
    rig.bus
        .request(StepSetting {
            target: SettingTarget::Range { channel: 1 },
            direction: 1,
        })
        .unwrap();
    rig.wait(|| rig.roll_items().len() == 3);

    rig.poll_samples(old, 0);
    rig.poll_samples(rig.setup_revision(), 0);
    rig.wait(|| rig.roll_items().len() == 4);

    let items = rig.roll_items();
    assert_eq!(kinds(&items)[3], "samples 0");
    assert_eq!(items[3].session, 2);
}

#[test]
fn given_roll_running_when_the_scope_is_unplugged_then_the_session_closes_as_disconnected() {
    let rig = start_rolling();

    rig.unplug();

    assert_eq!(kinds(&rig.roll_items()), ["opened", "closed Disconnected"]);
}

#[test]
fn given_roll_running_when_the_scope_returns_then_the_run_resumes_in_a_new_session() {
    let rig = start_rolling();
    rig.unplug();

    rig.plug();
    rig.wait(|| rig.roll_items().len() == 3);

    let items = rig.roll_items();
    assert_eq!(kinds(&items), ["opened", "closed Disconnected", "opened"]);
    assert_eq!(sessions(&items), [1, 1, 2]);
}
