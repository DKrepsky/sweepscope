mod common;

use std::sync::Arc;
use std::time::Duration;

use common::*;
use sweepscope_display::{
    DisplayChanged, DisplayDeps, DisplayFrameReady, DisplayModule, GetDisplay, InterpolationMode, Persistence,
    SetInterpolation, SetPersistence,
};
use sweepscope_kernel::contracts::{FrameCaptured, TriggerOutcome};
use sweepscope_kernel::Bus;

const WAIT: Duration = Duration::from_secs(2);

fn captured(level: u8) -> FrameCaptured {
    FrameCaptured {
        frame: one_channel(vec![level; 4], TriggerOutcome::None),
        context: context(0, 4),
        derived: None,
    }
}

fn start(bus: &Bus) -> DisplayModule {
    let module = DisplayModule::new(DisplayDeps {
        bus: bus.clone(),
        unix_time_us: || 5_000.0,
    })
    .unwrap();
    bus.request(SetInterpolation {
        mode: InterpolationMode::Linear,
    })
    .unwrap();
    module
}

#[test]
fn given_a_captured_frame_when_published_then_an_encoded_frame_follows_with_the_produced_time() {
    let bus = Bus::new();
    let ready = bus.subscribe::<DisplayFrameReady>();
    let _module = start(&bus);

    bus.publish(captured(128));

    let bytes = ready.recv_timeout(WAIT).expect("frame").bytes;
    assert_eq!(&bytes[0..4], b"SSF1");
    assert_eq!(f64_at(&bytes, 8), 5_000.0);
}

#[test]
fn given_several_frames_when_encoded_then_the_sequence_increases_by_one() {
    let bus = Bus::new();
    let ready = bus.subscribe::<DisplayFrameReady>();
    let _module = start(&bus);

    bus.publish(captured(128));
    let first = u32_at(&ready.recv_timeout(WAIT).unwrap().bytes, 4);
    bus.publish(captured(128));
    let second = u32_at(&ready.recv_timeout(WAIT).unwrap().bytes, 4);

    assert_eq!(second, first.wrapping_add(1));
}

#[test]
fn given_the_same_persistence_twice_when_set_then_one_change_is_published() {
    let bus = Bus::new();
    let changed = bus.subscribe::<DisplayChanged>();
    let _module = start(&bus);
    changed.drain();

    bus.request(SetPersistence { value: Persistence::S2 }).unwrap();
    bus.request(SetPersistence { value: Persistence::S2 }).unwrap();

    let first = changed.recv_timeout(WAIT).expect("a change").0;
    assert_eq!(first.persistence, Persistence::S2);
    assert!(changed.recv_timeout(Duration::from_millis(100)).is_none());
    assert_eq!(bus.request(GetDisplay).unwrap().persistence, Persistence::S2);
}

#[test]
fn given_a_persistence_change_when_set_then_no_frame_is_published() {
    let bus = Bus::new();
    let ready = bus.subscribe::<DisplayFrameReady>();
    let _module = start(&bus);
    bus.publish(captured(128));
    ready.recv_timeout(WAIT).expect("frame");

    bus.request(SetPersistence {
        value: Persistence::Infinite,
    })
    .unwrap();

    assert!(ready.recv_timeout(Duration::from_millis(100)).is_none());
}

#[test]
fn given_two_frames_before_the_module_reads_when_it_encodes_then_only_the_newer_is_encoded() {
    let bus = Bus::new();
    let ready = bus.subscribe::<DisplayFrameReady>();
    let module = start(&bus);
    for _ in 0..50 {
        bus.publish(captured(28));
    }
    bus.publish(captured(228));

    let mut last = None;
    while let Some(event) = ready.recv_timeout(Duration::from_millis(300)) {
        last = Some(event);
    }

    assert_eq!(f32s(&last.expect("a frame").bytes), vec![0.0; 4]);
    drop(module);
}

#[test]
fn given_the_module_is_dropped_when_a_frame_is_published_then_nothing_is_encoded() {
    let bus = Bus::new();
    let ready = bus.subscribe::<DisplayFrameReady>();
    drop(start(&bus));

    bus.publish(captured(128));

    assert!(ready.recv_timeout(Duration::from_millis(100)).is_none());
}

#[test]
fn given_a_published_frame_when_a_copy_is_stamped_then_the_shared_bytes_are_untouched() {
    let bus = Bus::new();
    let ready = bus.subscribe::<DisplayFrameReady>();
    let _module = start(&bus);
    bus.publish(captured(128));

    let published = ready.recv_timeout(WAIT).unwrap().bytes;
    let mut copy = Arc::unwrap_or_clone(published.clone());
    sweepscope_display::stamp_sent(&mut copy, 1.0);

    assert_eq!(f64_at(&published, 16), 0.0);
    assert_eq!(f64_at(&copy, 16), 1.0);
}

#[test]
fn given_a_running_module_when_dropped_then_both_workers_stop() {
    let bus = Bus::new();
    let module = start(&bus);
    let began = std::time::Instant::now();

    drop(module);

    assert!(began.elapsed() < WAIT);
}
