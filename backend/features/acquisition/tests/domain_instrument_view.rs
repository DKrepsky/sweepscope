mod domain_support;

use std::time::Duration;

use domain_support::{assert_close, capabilities, enable_all, patch_enabled, ready_instrument};
use sweepscope_acquisition::{CaptureMeter, ChannelPatch, Defaults, Instrument, InstrumentState, SettingTarget};
use sweepscope_kernel::testing::FakeClock;
use sweepscope_kernel::Clock;

const CH1: SettingTarget = SettingTarget::Range { channel: 1 };

fn state(instrument: &Instrument) -> InstrumentState {
    instrument.state(1).unwrap()
}

fn probe(factor: f64) -> ChannelPatch {
    ChannelPatch {
        probe: Some(factor),
        ..ChannelPatch::default()
    }
}

#[test]
fn given_the_default_range_when_the_state_is_read_then_the_label_reads_1_volt_per_div() {
    let s = state(&ready_instrument());

    let range = &s.channels[0].range;

    assert_eq!((range.label.text.as_str(), range.label.unit.as_str()), ("1", "V/div"));
}

#[test]
fn given_probe_10x_when_the_state_is_read_then_the_range_and_level_labels_are_multiplied_by_10() {
    let mut instrument = ready_instrument();
    instrument.set_trigger_level(2.0).unwrap();
    let plain = state(&instrument);

    instrument.set_channel(1, probe(10.0)).unwrap();
    let probed = state(&instrument);

    assert_close(
        probed.channels[0].range.label.value,
        plain.channels[0].range.label.value * 10.0,
    );
    assert_close(probed.trigger.level.value, plain.trigger.level.value * 10.0);
    assert_eq!(probed.channels[0].probe_text, "10x");
}

#[test]
fn given_probe_10x_when_the_setup_is_read_then_the_device_scale_and_level_are_unchanged() {
    let mut instrument = ready_instrument();
    instrument.set_trigger_level(2.0).unwrap();
    let before = instrument.setup().unwrap().clone();

    let change = instrument.set_channel(1, probe(10.0)).unwrap();

    assert_eq!(instrument.setup().unwrap(), &before);
    assert!(!change.setup_changed && change.state_changed);
}

#[test]
fn given_invert_when_toggled_then_the_state_and_frame_context_change_but_not_the_setup() {
    let mut instrument = ready_instrument();
    let before = instrument.setup().unwrap().clone();

    let change = instrument
        .set_channel(
            2,
            ChannelPatch {
                invert: Some(true),
                ..ChannelPatch::default()
            },
        )
        .unwrap();

    assert!(!change.setup_changed && change.state_changed);
    assert_eq!(instrument.setup().unwrap(), &before);
    assert_eq!(instrument.frame_context(7).invert, [false, true, false, false]);
    assert!(state(&instrument).channels[1].invert);
}

#[test]
fn given_a_window_of_10_divisions_when_the_state_is_read_then_the_window_text_names_the_record_size() {
    let s = state(&ready_instrument());

    assert_eq!(s.horizontal.window_text, "10.0 of 16.4 div");
    assert_close(s.horizontal.window_size_fraction, 2501.0 / 4096.0);
    assert_close(s.horizontal.trigger_tick_fraction, 0.5);
}

#[test]
fn given_the_default_trigger_when_the_tags_are_read_then_they_show_source_slope_level_and_position() {
    let s = state(&ready_instrument());

    assert_eq!(s.trigger.level_tag, "T1 \u{2197} 0.00 V");
    assert_eq!(s.trigger.position_tag, "50.0 %");
}

#[test]
fn given_a_ready_device_when_the_state_is_read_then_every_control_is_available_except_force_while_stopped() {
    let s = state(&ready_instrument());

    assert!(s.controls.run.enabled && s.controls.vertical.enabled);
    assert!(s.controls.horizontal.enabled && s.controls.trigger.enabled);
    assert!(!s.controls.force.enabled && s.controls.force.reason.is_some());
}

#[test]
fn given_a_running_scope_when_the_state_is_read_then_force_is_available() {
    let mut instrument = ready_instrument();

    instrument.set_running(true).unwrap();

    assert!(state(&instrument).controls.force.enabled);
}

#[test]
fn given_no_ready_device_when_the_state_is_read_then_every_availability_has_a_reason() {
    let mut instrument = ready_instrument();
    instrument.on_lost();

    let s = state(&instrument);

    let c = &s.controls;
    let mut all = vec![&c.run, &c.force, &c.vertical, &c.horizontal, &c.trigger];
    all.extend([&s.horizontal.time_base.faster, &s.horizontal.time_base.slower]);
    for channel in &s.channels {
        all.extend([&channel.enable, &channel.range.faster, &channel.range.slower]);
    }
    assert!(all
        .iter()
        .all(|a| !a.enabled && a.reason.as_deref() == Some("No scope connected")));
}

#[test]
fn given_a_connecting_device_when_the_state_is_read_then_the_reason_is_connecting() {
    let mut instrument = ready_instrument();

    instrument.set_unavailable("Connecting");

    assert_eq!(state(&instrument).controls.run.reason.as_deref(), Some("Connecting"));
}

#[test]
fn given_no_ready_device_when_an_edit_arrives_then_it_is_rejected_with_the_reason() {
    let mut instrument = ready_instrument();
    instrument.on_lost();

    let error = instrument.set_running(true).unwrap_err();

    assert_eq!(error.message, "No scope connected");
    assert!(instrument.step(CH1, 1).is_err());
}

#[test]
fn given_a_lost_device_when_the_state_is_read_then_the_last_setup_stays_and_running_is_false() {
    let mut instrument = ready_instrument();
    instrument.step(CH1, 1).unwrap();
    instrument.set_running(true).unwrap();
    let range = state(&instrument).channels[0].range.index;

    instrument.on_lost();

    let s = state(&instrument);
    assert_eq!(s.channels[0].range.index, range);
    assert!(!s.running);
}

#[test]
fn given_a_setup_before_a_loss_when_the_device_is_ready_again_then_the_restored_setup_is_the_same() {
    let mut instrument = ready_instrument();
    enable_all(&mut instrument);
    instrument.step(CH1, 1).unwrap();
    instrument.step(SettingTarget::TimeBase, 1).unwrap();
    let before = instrument.setup().unwrap().clone();

    instrument.on_lost();
    let restore = instrument.on_ready(capabilities());

    assert_eq!(restore.setup, before);
    assert_eq!(instrument.setup().unwrap(), &before);
}

#[test]
fn given_a_run_in_progress_at_the_loss_when_the_device_is_ready_again_then_the_restore_resumes_it() {
    let mut instrument = ready_instrument();
    instrument.set_running(true).unwrap();
    instrument.on_lost();

    let restore = instrument.on_ready(capabilities());

    assert!(restore.resume);
    assert!(state(&instrument).running);
}

#[test]
fn given_a_stopped_scope_at_the_loss_when_the_device_is_ready_again_then_it_stays_stopped() {
    let mut instrument = ready_instrument();
    instrument.on_lost();

    let restore = instrument.on_ready(capabilities());

    assert!(!restore.resume);
    assert!(!state(&instrument).running);
}

#[test]
fn given_resume_disabled_when_the_device_is_ready_again_then_a_run_in_progress_does_not_resume() {
    let defaults = Defaults {
        resume_after_reconnect: false,
        ..Defaults::default()
    };
    let mut instrument = Instrument::new(defaults);
    instrument.on_ready(capabilities());
    instrument.set_running(true).unwrap();
    instrument.on_lost();

    assert!(!instrument.on_ready(capabilities()).resume);
}

#[test]
fn given_a_new_device_with_fewer_ranges_when_ready_then_the_setup_indexes_and_positions_are_revalidated() {
    let mut instrument = ready_instrument();
    for _ in 0..3 {
        instrument.step(CH1, 1).unwrap();
    }
    instrument.on_lost();
    let mut smaller = capabilities();
    smaller.vertical_ranges.truncate(4);

    let restore = instrument.on_ready(smaller);

    assert_eq!(restore.setup.channels[0].range_index, 3);
}

#[test]
fn given_the_default_setup_when_the_frame_context_is_read_then_it_carries_the_window_and_revision() {
    let mut instrument = ready_instrument();

    let context = instrument.frame_context(9);

    assert_eq!(
        (context.setup_revision, context.window_start, context.window_len),
        (9, 798, 2501)
    );
    instrument.step(SettingTarget::TimeBase, 1).unwrap();
    assert_eq!(instrument.frame_context(10).window_len, 2501);
}

#[test]
fn given_a_probe_when_the_frame_context_is_read_then_it_carries_the_probe_the_time_base_and_no_calibration() {
    let mut instrument = ready_instrument();
    instrument.set_channel(2, probe(10.0)).unwrap();
    instrument.step(CH1, 1).unwrap();
    instrument.step(CH1, 1).unwrap();
    instrument.step(CH1, 1).unwrap();
    instrument.step(CH1, 1).unwrap();

    let context = instrument.frame_context(1);

    assert_eq!(context.probe, [1.0, 10.0, 1.0, 1.0]);
    assert_eq!(context.calibrated, [false; 4]);
    assert_close(context.seconds_per_div, 200e-6);
}

#[test]
fn given_no_device_ever_ready_when_the_frame_context_is_read_then_the_new_fields_are_neutral() {
    let context = Instrument::new(Defaults::default()).frame_context(1);

    assert_eq!((context.probe, context.calibrated), ([1.0; 4], [false; 4]));
    assert_eq!(context.seconds_per_div, 0.0);
}

#[test]
fn given_a_ready_device_when_the_state_is_read_then_files_follow_the_connection_and_there_is_no_notice() {
    let instrument = ready_instrument();

    let state = state(&instrument);

    assert!(state.controls.files.enabled);
    assert_eq!(state.setup_notice, None);
}

#[test]
fn given_the_default_setup_when_the_facts_are_read_then_they_show_rate_interval_and_record_points() {
    let instrument = ready_instrument();
    let clock = FakeClock::default();

    let facts = instrument.facts(3, &CaptureMeter::default(), clock.now()).unwrap();

    assert_eq!(facts.revision, 3);
    assert_eq!(facts.sample_rate.unwrap().text, "1.25");
    assert_eq!(facts.sample_interval.unwrap().unit, "ns");
    assert_eq!(facts.record_points.unwrap().text, "4.10");
    assert!(facts.capture_rate.is_none() && facts.dead_time.is_none());
}

#[test]
fn given_measured_frames_when_the_facts_are_read_then_capture_rate_and_dead_time_are_set() {
    let instrument = ready_instrument();
    let clock = FakeClock::default();
    let mut meter = CaptureMeter::default();
    for _ in 0..100 {
        meter.record(clock.now());
        clock.sleep(Duration::from_millis(50));
    }

    let facts = instrument.facts(1, &meter, clock.now()).unwrap();

    assert_eq!(facts.capture_rate.unwrap().unit, "Hz");
    assert_eq!(facts.dead_time.unwrap().unit, "%");
}

#[test]
fn given_the_enabled_channel_count_when_it_rises_then_the_sample_rate_text_changes() {
    let mut instrument = ready_instrument();
    for _ in 0..40 {
        instrument.step(SettingTarget::TimeBase, -1).unwrap();
    }
    let clock = FakeClock::default();
    let one = instrument.facts(1, &CaptureMeter::default(), clock.now()).unwrap();

    instrument.set_channel(2, patch_enabled(true)).unwrap();
    let two = instrument.facts(2, &CaptureMeter::default(), clock.now()).unwrap();

    assert_ne!(one.sample_rate.unwrap().text, two.sample_rate.unwrap().text);
}
