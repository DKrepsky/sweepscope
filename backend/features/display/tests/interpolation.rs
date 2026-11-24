mod common;

use std::time::Duration;

use common::*;
use sweepscope_display::{
    DisplayChanged, DisplayDeps, DisplayFrameReady, DisplayModule, DisplayState, GetDisplay, InterpolationMode,
    SetInterpolation,
};
use sweepscope_kernel::contracts::{FrameCaptured, TriggerOutcome};
use sweepscope_kernel::{Bus, Subscription};

const WAIT: Duration = Duration::from_secs(2);
const SAMPLE_INTERVAL_S: f64 = 2e-9;
const HIGH_V: f64 = 1.91;
const RAMP_S: f64 = 250e-9 / 0.8;
const VOLTS_PER_DIV: f64 = 0.5;

struct Rig {
    bus: Bus,
    ready: Subscription<DisplayFrameReady>,
    changed: Subscription<DisplayChanged>,
    _module: DisplayModule,
}

fn rig() -> Rig {
    let bus = Bus::new();
    let ready = bus.subscribe::<DisplayFrameReady>();
    let changed = bus.subscribe::<DisplayChanged>();
    let module = DisplayModule::new(DisplayDeps {
        bus: bus.clone(),
        unix_time_us: || 0.0,
    })
    .unwrap();
    Rig {
        bus,
        ready,
        changed,
        _module: module,
    }
}

fn flat(window_len: usize, trigger: TriggerOutcome) -> FrameCaptured {
    FrameCaptured {
        frame: one_channel(vec![128; window_len + 10], trigger),
        context: context(5, window_len),
        derived: None,
    }
}

fn next_frame(rig: &Rig) -> Vec<u8> {
    Vec::clone(&rig.ready.recv_timeout(WAIT).expect("frame").bytes)
}

fn points(bytes: &[u8]) -> u32 {
    u32_at(bytes, 28)
}

fn mode_of(bus: &Bus) -> DisplayState {
    bus.request(GetDisplay).unwrap()
}

#[test]
fn given_automatic_mode_and_5_samples_per_division_when_a_frame_arrives_then_it_carries_2501_points_and_the_interval_is_divided(
) {
    let rig = rig();

    rig.bus.publish(flat(51, TriggerOutcome::None));
    let bytes = next_frame(&rig);

    assert_eq!(points(&bytes), 2501);
    assert_eq!(f64_at(&bytes, 32), 4e-6 / 50.0);
    assert_eq!(f32s(&bytes).len(), 2501);
}

#[test]
fn given_automatic_mode_and_250_samples_per_division_when_a_frame_arrives_then_the_original_points_are_sent() {
    let rig = rig();

    rig.bus.publish(flat(2501, TriggerOutcome::None));
    let bytes = next_frame(&rig);

    assert_eq!(points(&bytes), 2501);
    assert_eq!(f64_at(&bytes, 32), 4e-6);
    assert_eq!(mode_of(&rig.bus).interpolation, InterpolationMode::Linear);
}

#[test]
fn given_automatic_mode_when_a_short_window_arrives_then_the_effective_mode_is_sinc_and_still_automatic() {
    let rig = rig();

    rig.bus.publish(flat(51, TriggerOutcome::None));
    next_frame(&rig);

    let state = mode_of(&rig.bus);
    assert_eq!(state.interpolation, InterpolationMode::Sinc);
    assert!(state.automatic);
}

#[test]
fn given_no_frame_yet_when_the_display_is_read_then_it_is_automatic_linear() {
    let rig = rig();

    let state = mode_of(&rig.bus);

    assert_eq!(state.interpolation, InterpolationMode::Linear);
    assert!(state.automatic);
    assert_eq!(state.revision, 0);
}

#[test]
fn given_the_user_picks_dots_at_a_fast_time_base_when_a_frame_arrives_then_the_original_window_is_sent_and_automatic_is_false(
) {
    let rig = rig();
    rig.bus
        .request(SetInterpolation {
            mode: InterpolationMode::Dots,
        })
        .unwrap();

    rig.bus.publish(flat(51, TriggerOutcome::None));
    let bytes = next_frame(&rig);

    assert_eq!(points(&bytes), 51);
    let state = mode_of(&rig.bus);
    assert_eq!(state.interpolation, InterpolationMode::Dots);
    assert!(!state.automatic);
}

#[test]
fn given_a_stopped_acquisition_when_the_mode_changes_then_the_last_frame_is_published_again_in_the_new_mode() {
    let rig = rig();
    rig.bus.publish(flat(51, TriggerOutcome::None));
    assert_eq!(points(&next_frame(&rig)), 2501);

    rig.bus
        .request(SetInterpolation {
            mode: InterpolationMode::Linear,
        })
        .unwrap();

    assert_eq!(points(&next_frame(&rig)), 51);
}

#[test]
fn given_no_frame_yet_when_the_mode_is_set_then_nothing_is_published() {
    let rig = rig();

    rig.bus
        .request(SetInterpolation {
            mode: InterpolationMode::Dots,
        })
        .unwrap();

    assert!(rig.ready.recv_timeout(Duration::from_millis(100)).is_none());
}

#[test]
fn given_the_same_effective_mode_twice_when_chosen_then_one_display_changed_is_published() {
    let rig = rig();
    rig.bus.publish(flat(2501, TriggerOutcome::None));
    next_frame(&rig);

    for _ in 0..2 {
        rig.bus
            .request(SetInterpolation {
                mode: InterpolationMode::Linear,
            })
            .unwrap();
    }

    let events = rig.changed.drain();
    assert_eq!(events.len(), 1);
    assert!(!events[0].0.automatic);
}

#[test]
fn given_frames_in_the_same_automatic_mode_when_they_arrive_then_display_changed_is_published_once() {
    let rig = rig();

    for _ in 0..3 {
        rig.bus.publish(flat(51, TriggerOutcome::None));
        next_frame(&rig);
    }

    let events = rig.changed.drain();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].0.interpolation, InterpolationMode::Sinc);
    assert_eq!(events[0].0.revision, 1);
}

#[test]
fn given_a_trigger_sample_when_the_window_is_upsampled_then_its_index_is_multiplied_by_the_factor() {
    let rig = rig();

    rig.bus
        .publish(flat(51, TriggerOutcome::Triggered { sample_index: 25 }));
    let bytes = next_frame(&rig);

    assert_eq!(i32_at(&bytes, 40), 20 * 50);
}

#[test]
fn given_a_trigger_cropped_away_when_the_window_is_upsampled_then_there_is_no_trigger_index() {
    let rig = rig();

    rig.bus.publish(flat(51, TriggerOutcome::Triggered { sample_index: 0 }));

    assert_eq!(i32_at(&next_frame(&rig), 40), -1);
}

#[test]
fn given_a_window_of_one_point_when_in_sinc_mode_then_it_is_sent_as_is() {
    let rig = rig();
    rig.bus
        .request(SetInterpolation {
            mode: InterpolationMode::Sinc,
        })
        .unwrap();

    rig.bus.publish(flat(1, TriggerOutcome::None));

    assert_eq!(points(&next_frame(&rig)), 1);
}

fn calibrator_level(t: f64) -> f64 {
    (0.5 + t / RAMP_S).clamp(0.0, 1.0)
}

fn calibrator_frame(window_len: usize) -> FrameCaptured {
    let centre = 500usize;
    let codes: Vec<u8> = (0..2 * centre + 1)
        .map(|i| {
            let t = (i as f64 - centre as f64) * SAMPLE_INTERVAL_S;
            let volts = HIGH_V * calibrator_level(t);
            (128.0 + volts / VOLTS_PER_DIV * COUNTS_PER_DIV).round() as u8
        })
        .collect();
    let mut frame = frame([Some(channel(codes, 0.0)), None, None, None], TriggerOutcome::None);
    frame.sample_interval_s = SAMPLE_INTERVAL_S;
    FrameCaptured {
        frame: std::sync::Arc::new(frame),
        context: context(centre - window_len / 2, window_len),
        derived: None,
    }
}

/// Time between the 20 % and 80 % crossings of the edge, scaled to 10 to 90 %: the 20 ns/div window (200 ns) is shorter than the edge.
fn rise_time_s(bytes: &[u8]) -> f64 {
    let low = 4.0;
    let high_code = (128.0 + HIGH_V / VOLTS_PER_DIV * COUNTS_PER_DIV).round();
    let high = 4.0 - (high_code - 128.0) / COUNTS_PER_DIV;
    let values = f32s(bytes);
    let crossing = |fraction: f64| {
        let level = low + (high - low) * fraction;
        let at = values
            .windows(2)
            .position(|pair| f64::from(pair[0]) > level && f64::from(pair[1]) <= level);
        let at = at.expect("the trace crosses the level");
        let (a, b) = (f64::from(values[at]), f64::from(values[at + 1]));
        at as f64 + (a - level) / (a - b)
    };
    let samples = crossing(0.8) - crossing(0.2);
    samples * f64_at(bytes, 32) * (0.8 / 0.6)
}

fn rise_time_at(seconds_per_div: f64) -> f64 {
    let window_len = (10.0 * seconds_per_div / SAMPLE_INTERVAL_S).round() as usize + 1;
    let rig = rig();
    rig.bus.publish(calibrator_frame(window_len));
    rise_time_s(&next_frame(&rig))
}

#[test]
fn given_the_calibrator_at_20_ns_per_div_when_encoded_then_the_rise_time_is_240_to_256_ns() {
    let rise = rise_time_at(20e-9);

    assert!((240e-9..=256e-9).contains(&rise), "{rise}");
}

#[test]
fn given_the_calibrator_at_50_ns_per_div_when_encoded_then_the_rise_time_is_240_to_256_ns() {
    let rise = rise_time_at(50e-9);

    assert!((240e-9..=256e-9).contains(&rise), "{rise}");
}

#[test]
fn given_the_calibrator_at_100_ns_per_div_when_encoded_then_the_rise_time_is_240_to_256_ns() {
    let rise = rise_time_at(100e-9);

    assert!((240e-9..=256e-9).contains(&rise), "{rise}");
}
