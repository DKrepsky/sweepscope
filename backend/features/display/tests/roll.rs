mod common;

use std::time::Duration;

use common::*;
use sweepscope_display::{DisplayDeps, DisplayModule, DisplayRollItem};
use sweepscope_kernel::contracts::{Frame, RollCaptured, RollCloseReason, RollContext, RollItem};
use sweepscope_kernel::{Bus, Subscription};

const WAIT: Duration = Duration::from_secs(2);
const HEADER_LEN: usize = 32;

struct Rig {
    bus: Bus,
    items: Subscription<DisplayRollItem>,
    _module: DisplayModule,
}

fn rig() -> Rig {
    let bus = Bus::new();
    let items = bus.subscribe::<DisplayRollItem>();
    let module = DisplayModule::new(DisplayDeps {
        bus: bus.clone(),
        unix_time_us: || 0.0,
    })
    .unwrap();
    Rig {
        bus,
        items,
        _module: module,
    }
}

fn roll_context(session: u32) -> RollContext {
    RollContext {
        session,
        invert: [false; 4],
        probe: [1.0; 4],
        seconds_per_div: 1.0,
    }
}

fn captured(session: u32, item: RollItem) -> RollCaptured {
    RollCaptured {
        session,
        item,
        context: roll_context(session),
    }
}

fn samples(first_index: u64, codes: Vec<u8>) -> RollItem {
    RollItem::Samples {
        first_index,
        frame: one_channel(codes, sweepscope_kernel::contracts::TriggerOutcome::None),
    }
}

impl Rig {
    fn next(&self) -> Vec<u8> {
        self.items.recv_timeout(WAIT).expect("a roll item").bytes.to_vec()
    }
}

fn kind(bytes: &[u8]) -> u8 {
    bytes[8]
}

fn u64_at(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}

fn values(bytes: &[u8]) -> Vec<f32> {
    let (chunks, _) = bytes[HEADER_LEN..].as_chunks::<4>();
    chunks.iter().map(|c| f32::from_le_bytes(*c)).collect()
}

#[test]
fn given_samples_when_encoded_then_the_header_and_divisions_follow_the_layout() {
    let rig = rig();

    rig.bus.publish(captured(3, samples(0, vec![128, 228, 28])));

    let bytes = rig.next();
    assert_eq!(&bytes[0..4], b"SSL1");
    assert_eq!(kind(&bytes), 0);
    assert_eq!(bytes[9], 0b0001);
    assert_eq!(u32_at(&bytes, 12), 3);
    assert_eq!(u64_at(&bytes, 16), 0);
    assert_eq!(u32_at(&bytes, 28), 3);
    assert_eq!(values(&bytes), vec![4.0, 0.0, 8.0]);
}

#[test]
fn given_a_session_when_items_follow_then_sequence_numbers_rise_by_one() {
    let rig = rig();

    rig.bus.publish(captured(1, RollItem::Opened));
    rig.bus.publish(captured(1, samples(0, vec![128; 2])));
    rig.bus.publish(captured(1, samples(2, vec![128; 2])));
    rig.bus.publish(captured(
        1,
        RollItem::Closed {
            reason: RollCloseReason::Stopped,
        },
    ));

    let sequences: Vec<u32> = (0..3).map(|_| u32_at(&rig.next(), 4)).collect();
    assert_eq!(sequences, vec![0, 1, 2]);
}

#[test]
fn given_an_index_jump_when_samples_arrive_then_a_gap_item_precedes_them() {
    let rig = rig();
    rig.bus.publish(captured(1, samples(0, vec![128; 4])));
    rig.next();

    rig.bus.publish(captured(1, samples(10, vec![128; 4])));

    let gap = rig.next();
    assert_eq!(kind(&gap), 1);
    assert_eq!(u64_at(&gap, 16), 3);
    assert_eq!(u32_at(&gap, 24), 6);
    assert_eq!(kind(&rig.next()), 0);
}

#[test]
fn given_a_gap_item_when_samples_follow_after_it_then_no_second_gap_is_added() {
    let rig = rig();
    rig.bus.publish(captured(1, samples(0, vec![128; 4])));
    rig.bus.publish(captured(1, RollItem::Gap { after: 3, missed: 6 }));
    rig.bus.publish(captured(1, samples(10, vec![128; 4])));

    let kinds: Vec<u8> = (0..3).map(|_| kind(&rig.next())).collect();

    assert_eq!(kinds, vec![0, 1, 0]);
    assert!(rig.items.recv_timeout(Duration::from_millis(100)).is_none());
}

#[test]
fn given_a_new_session_when_samples_start_at_zero_then_no_gap_is_added() {
    let rig = rig();
    rig.bus.publish(captured(1, samples(0, vec![128; 8])));
    rig.next();

    rig.bus.publish(captured(2, RollItem::Opened));
    rig.bus.publish(captured(2, samples(0, vec![128; 8])));

    let bytes = rig.next();
    assert_eq!(kind(&bytes), 0);
    assert_eq!(u32_at(&bytes, 28), 2);
}

#[test]
fn given_a_closed_item_when_encoded_then_the_reason_code_is_set() {
    let rig = rig();

    rig.bus.publish(captured(
        1,
        RollItem::Closed {
            reason: RollCloseReason::SetupChanged,
        },
    ));

    let bytes = rig.next();
    assert_eq!(kind(&bytes), 2);
    assert_eq!(bytes[10], 3);
    assert_eq!(bytes.len(), HEADER_LEN);
}

#[test]
fn given_an_inverted_channel_when_encoded_then_the_roll_samples_mirror_around_ground() {
    let rig = rig();
    let mut item = captured(1, samples(0, vec![153]));
    item.context.invert[0] = true;

    rig.bus.publish(item);

    assert_eq!(values(&rig.next()), vec![5.0]);
}

fn golden_items() -> Vec<RollCaptured> {
    let frame: Frame = frame(
        [
            Some(channel(vec![28, 78, 128], 0.0)),
            None,
            Some(channel(vec![178, 203, 153], 2.0)),
            None,
        ],
        sweepscope_kernel::contracts::TriggerOutcome::None,
    );
    let mut inverted = captured(
        7,
        RollItem::Samples {
            first_index: 0,
            frame: frame.clone().into(),
        },
    );
    inverted.context.invert[2] = true;
    vec![
        captured(7, RollItem::Opened),
        inverted,
        captured(7, RollItem::Gap { after: 2, missed: 5 }),
        captured(
            7,
            RollItem::Samples {
                first_index: 8,
                frame: frame.into(),
            },
        ),
        captured(
            7,
            RollItem::Closed {
                reason: RollCloseReason::LeftRoll,
            },
        ),
    ]
}

#[test]
fn given_the_golden_roll_items_when_encoded_then_the_bytes_equal_the_committed_fixture() {
    let rig = rig();
    for item in golden_items() {
        rig.bus.publish(item);
    }

    let bytes: Vec<u8> = (0..4).flat_map(|_| rig.next()).collect();

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/roll-v1.bin");
    if std::env::var_os("SWEEPSCOPE_UPDATE_FIXTURES").is_some() {
        std::fs::write(&path, &bytes).unwrap();
    }
    let committed = std::fs::read(&path).expect("fixture missing; run with SWEEPSCOPE_UPDATE_FIXTURES=1");
    assert_eq!(bytes, committed);
}
