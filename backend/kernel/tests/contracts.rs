use std::sync::Arc;
use std::time::Duration;

use sweepscope_kernel::contracts::{
    AcquisitionSetup, ApplySetup, CaptureCompleted, CaptureOutcome, ChannelSetup, ConnectionChanged,
    ConnectionSnapshot, ConnectionState, Coupling, ForceTrigger, Frame, FrameCaptured, FrameContext, SetCapturing,
    Slope, Sweep, TriggerKind, TriggerOutcome, TriggerSetup,
};
use sweepscope_kernel::Bus;

fn frame() -> Arc<Frame> {
    Arc::new(Frame {
        channels: [None, None, None, None],
        counts_per_div: 24.0,
        sample_interval_s: 1e-6,
        trigger: TriggerOutcome::Auto,
    })
}

fn context(setup_revision: u64) -> FrameContext {
    FrameContext {
        setup_revision,
        window_start: 0,
        window_len: 2,
        invert: [false; 4],
        probe: [1.0; 4],
        range_verified: [true; 4],
        seconds_per_div: 1e-3,
    }
}

fn setup() -> AcquisitionSetup {
    let channel = ChannelSetup {
        enabled: true,
        range_index: 8,
        position_divs: 0.0,
        coupling: Coupling::Dc,
        bandwidth_limit: false,
    };
    AcquisitionSetup {
        channels: [channel.clone(), channel.clone(), channel.clone(), channel],
        time_base_index: 15,
        trigger: TriggerSetup {
            source: 0,
            level_volts: 0.0,
            slope: Slope::Rising,
            sweep: Sweep::Auto,
            horizontal_position_percent: 50.0,
            kind: TriggerKind::Edge,
        },
    }
}

#[test]
fn given_a_frame_subscriber_when_two_frames_are_captured_then_the_latest_one_wins_with_its_context() {
    let bus = Bus::new();
    let frames = bus.subscribe_latest::<FrameCaptured>();

    bus.publish(FrameCaptured {
        frame: frame(),
        context: context(1),
        derived: None,
    });
    bus.publish(FrameCaptured {
        frame: frame(),
        context: context(2),
        derived: None,
    });

    let newest = frames.recv_timeout(Duration::ZERO).unwrap();
    assert_eq!(newest.context.setup_revision, 2);
    assert_eq!(frames.lagged(), 1);
}

#[test]
fn given_a_capture_consumer_when_a_timeout_and_a_frame_complete_then_both_outcomes_arrive_in_order() {
    let bus = Bus::new();
    let completed = bus.subscribe::<CaptureCompleted>();

    bus.publish(CaptureCompleted {
        setup_revision: 3,
        outcome: CaptureOutcome::TimedOut,
    });
    bus.publish(CaptureCompleted {
        setup_revision: 3,
        outcome: CaptureOutcome::Frame(frame()),
    });

    let outcomes: Vec<bool> = completed
        .drain()
        .iter()
        .map(|event| matches!(event.outcome, CaptureOutcome::TimedOut))
        .collect();
    assert_eq!(outcomes, vec![true, false]);
}

#[test]
fn given_a_device_handler_when_the_three_requests_are_sent_then_each_returns_unit() {
    let bus = Bus::new();
    bus.handle(|request: ApplySetup| {
        assert_eq!(request.setup_revision, 7);
        Ok(())
    })
    .unwrap();
    bus.handle(|SetCapturing { on }| {
        assert!(on);
        Ok(())
    })
    .unwrap();
    bus.handle(|_: ForceTrigger| Ok(())).unwrap();

    assert_eq!(
        bus.request(ApplySetup {
            setup_revision: 7,
            setup: setup()
        }),
        Ok(())
    );
    assert_eq!(bus.request(SetCapturing { on: true }), Ok(()));
    assert_eq!(bus.request(ForceTrigger), Ok(()));
}

#[test]
fn given_a_connection_subscriber_when_the_state_changes_then_it_receives_the_snapshot_without_capabilities() {
    let bus = Bus::new();
    let changes = bus.subscribe::<ConnectionChanged>();

    bus.publish(ConnectionChanged {
        snapshot: ConnectionSnapshot {
            revision: 1,
            state: ConnectionState::Disconnected,
            source: None,
            identity: None,
            issue: None,
            simulator_active: false,
        },
        capabilities: None,
    });

    let change = changes.try_next().unwrap();
    assert_eq!(change.snapshot.state, ConnectionState::Disconnected);
    assert!(change.capabilities.is_none());
}
