use std::sync::Arc;
use std::time::Duration;

use sweepscope_kernel::contracts::{
    DerivedTrace, DerivedUnit, Frame, FrameCaptured, FrameContext, RollCaptured, RollCloseReason, RollContext,
    RollItem, RollPolled, RollPolledItem, TriggerOutcome,
};
use sweepscope_kernel::Bus;

fn frame() -> Arc<Frame> {
    Arc::new(Frame {
        channels: [None, None, None, None],
        counts_per_div: 24.0,
        sample_interval_s: 1e-3,
        trigger: TriggerOutcome::None,
    })
}

fn roll_context(session: u32) -> RollContext {
    RollContext {
        session,
        invert: [false; 4],
        probe: [1.0; 4],
        seconds_per_div: 0.5,
    }
}

#[test]
fn given_a_frame_with_a_derived_trace_when_published_then_the_subscriber_gets_the_same_trace() {
    let bus = Bus::new();
    let frames = bus.subscribe_latest::<FrameCaptured>();
    let trace = Arc::new(DerivedTrace {
        volts: vec![0.5, -0.5],
        unit: DerivedUnit::Volts,
        volts_per_div: 1.0,
        position_divs: 0.0,
    });

    bus.publish(FrameCaptured {
        frame: frame(),
        context: FrameContext {
            setup_revision: 1,
            window_start: 0,
            window_len: 2,
            invert: [false; 4],
            probe: [1.0; 4],
            range_verified: [true; 4],
            seconds_per_div: 1e-3,
        },
        derived: Some(Arc::clone(&trace)),
    });

    let received = frames.recv_timeout(Duration::ZERO).unwrap();
    assert_eq!(received.derived.as_deref(), Some(trace.as_ref()));
}

#[test]
fn given_roll_items_when_published_in_order_then_a_subscriber_sees_every_item_with_its_context() {
    let bus = Bus::new();
    let items = bus.subscribe::<RollCaptured>();
    let sent = [
        RollItem::Opened,
        RollItem::Samples {
            first_index: 0,
            frame: frame(),
        },
        RollItem::Gap { after: 24, missed: 100 },
        RollItem::Closed {
            reason: RollCloseReason::Stopped,
        },
    ];

    for item in sent.clone() {
        bus.publish(RollCaptured {
            session: 3,
            item,
            context: roll_context(3),
        });
    }

    let received: Vec<RollItem> = items.drain().into_iter().map(|captured| captured.item).collect();
    assert_eq!(received, sent);
}

#[test]
fn given_a_polled_gap_when_published_then_the_acquisition_side_receives_its_revision_and_extent() {
    let bus = Bus::new();
    let polled = bus.subscribe::<RollPolled>();

    bus.publish(RollPolled {
        setup_revision: 9,
        item: RollPolledItem::Gap { after: 10, missed: 5 },
    });

    let event = polled.try_next().unwrap();
    assert_eq!(event.setup_revision, 9);
    assert_eq!(event.item, RollPolledItem::Gap { after: 10, missed: 5 });
}
