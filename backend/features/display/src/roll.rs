//! `SSL1` roll items of ADR 0017 and the consumer rule that turns lost items into gaps.
use sweepscope_kernel::contracts::{Frame, RollCaptured, RollCloseReason, RollContext, RollItem};

use crate::convert::{scale_window, Window};

const MAGIC: [u8; 4] = *b"SSL1";
const HEADER_LEN: usize = 32;
const KIND_SAMPLES: u8 = 0;
const KIND_GAP: u8 = 1;
const KIND_CLOSED: u8 = 2;

/// Encodes the items of one roll stream: owns the expected sample index and the stream sequence number.
#[derive(Debug, Default)]
pub(crate) struct RollEncoder {
    session: Option<u32>,
    expected_index: u64,
    sequence: u32,
}

impl RollEncoder {
    /// Encoded items for one event, in stream order; `Opened` only restarts the index count.
    pub fn accept(&mut self, captured: &RollCaptured) -> Vec<Vec<u8>> {
        self.follow_session(captured.session);
        match &captured.item {
            RollItem::Opened => Vec::new(),
            RollItem::Samples { first_index, frame } => self.samples(captured, *first_index, frame),
            RollItem::Gap { after, missed } => {
                self.expected_index = after.saturating_add(1).saturating_add(*missed);
                vec![self.item(KIND_GAP, 0, captured.session, Body::gap(*after, *missed))]
            }
            RollItem::Closed { reason } => {
                vec![self.item(KIND_CLOSED, close_code(*reason), captured.session, Body::empty())]
            }
        }
    }

    fn follow_session(&mut self, session: u32) {
        if self.session != Some(session) {
            self.session = Some(session);
            self.expected_index = 0;
        }
    }

    fn samples(&mut self, captured: &RollCaptured, first_index: u64, frame: &Frame) -> Vec<Vec<u8>> {
        let session = captured.session;
        let mut items = Vec::new();
        if first_index > self.expected_index {
            let after = self.expected_index.saturating_sub(1);
            let missed = first_index - self.expected_index;
            items.push(self.item(KIND_GAP, 0, session, Body::gap(after, missed)));
        }
        let body = Body::samples(frame, &captured.context, first_index);
        self.expected_index = first_index.saturating_add(u64::from(body.count));
        items.push(self.item(KIND_SAMPLES, 0, session, body));
        items
    }

    fn item(&mut self, kind: u8, close_reason: u8, session: u32, body: Body) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(HEADER_LEN + body.values.len() * 4);
        bytes.extend_from_slice(&MAGIC);
        bytes.extend_from_slice(&self.sequence.to_le_bytes());
        bytes.extend_from_slice(&[kind, body.mask, close_reason, 0]);
        bytes.extend_from_slice(&body.count.to_le_bytes());
        bytes.extend_from_slice(&body.index.to_le_bytes());
        bytes.extend_from_slice(&body.missed.to_le_bytes());
        bytes.extend_from_slice(&session.to_le_bytes());
        for value in &body.values {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        self.sequence = self.sequence.wrapping_add(1);
        bytes
    }
}

struct Body {
    mask: u8,
    count: u32,
    index: u64,
    missed: u32,
    values: Vec<f32>,
}

impl Body {
    fn empty() -> Self {
        Self {
            mask: 0,
            count: 0,
            index: 0,
            missed: 0,
            values: Vec::new(),
        }
    }

    fn gap(after: u64, missed: u64) -> Self {
        Self {
            index: after,
            missed: u32::try_from(missed).unwrap_or(u32::MAX),
            ..Self::empty()
        }
    }

    fn samples(frame: &Frame, context: &RollContext, first_index: u64) -> Self {
        let present: Vec<_> = frame
            .channels
            .iter()
            .enumerate()
            .filter_map(|(index, data)| data.as_ref().map(|data| (index, data)))
            .collect();
        let len = present.first().map_or(0, |(_, data)| data.codes.len());
        let window = Window::within(len, 0, len);
        let values = present
            .iter()
            .flat_map(|(index, data)| scale_window(data, frame.counts_per_div, window, context.invert[*index]))
            .collect();
        Self {
            mask: present.iter().fold(0, |mask, (index, _)| mask | 1 << index),
            count: u32::try_from(len).unwrap_or(u32::MAX),
            index: first_index,
            missed: 0,
            values,
        }
    }
}

fn close_code(reason: RollCloseReason) -> u8 {
    match reason {
        RollCloseReason::Stopped => 0,
        RollCloseReason::LeftRoll => 1,
        RollCloseReason::Disconnected => 2,
        RollCloseReason::SetupChanged => 3,
    }
}
