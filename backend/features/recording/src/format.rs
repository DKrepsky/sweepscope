//! `.ssroll` v1 byte layout (docs/formats/roll-record-v1.md); the only place that knows it.
use serde::Serialize;
use sweepscope_kernel::contracts::{Frame, RollCloseReason, RollContext};

const MAGIC: &[u8; 4] = b"SSRL";
const VERSION: u16 = 1;

const SEGMENT: u8 = 1;
const SAMPLES: u8 = 2;
const GAP: u8 = 3;
const END: u8 = 4;

/// Why a recording ended without a failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndReason {
    Stopped,
    LeftRoll,
    Disconnected,
}

impl EndReason {
    /// `None` for a close that only replaces the session: the recording goes on in a new segment.
    pub fn of_close(reason: RollCloseReason) -> Option<Self> {
        match reason {
            RollCloseReason::Stopped => Some(Self::Stopped),
            RollCloseReason::LeftRoll => Some(Self::LeftRoll),
            RollCloseReason::Disconnected => Some(Self::Disconnected),
            RollCloseReason::SetupChanged => None,
        }
    }

    fn code(self) -> u8 {
        match self {
            Self::Stopped => 0,
            Self::LeftRoll => 1,
            Self::Disconnected => 2,
        }
    }
}

/// Everything that must stay equal inside one segment; a difference starts the next segment.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SegmentSetup {
    pub session: u32,
    pub sample_interval_s: f64,
    pub seconds_per_div: f64,
    pub counts_per_div: f64,
    pub channels: Vec<ChannelSetup>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChannelSetup {
    pub channel: u8,
    pub volts_per_div: f64,
    pub probe: f64,
    pub zero_code: f64,
    pub inverted: bool,
}

impl SegmentSetup {
    pub fn of(context: &RollContext, frame: &Frame) -> Self {
        let channels = frame
            .channels
            .iter()
            .enumerate()
            .filter_map(|(index, data)| {
                let data = data.as_ref()?;
                Some(ChannelSetup {
                    channel: u8::try_from(index + 1).unwrap_or(u8::MAX),
                    volts_per_div: data.volts_per_div,
                    probe: context.probe[index],
                    zero_code: data.zero_code,
                    inverted: context.invert[index],
                })
            })
            .collect();
        Self {
            session: context.session,
            sample_interval_s: frame.sample_interval_s,
            seconds_per_div: context.seconds_per_div,
            counts_per_div: frame.counts_per_div,
            channels,
        }
    }
}

#[derive(Serialize)]
struct SegmentJson<'a> {
    started_unix_us: f64,
    first_index: u64,
    #[serde(flatten)]
    setup: &'a SegmentSetup,
}

pub fn header() -> Vec<u8> {
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(&VERSION.to_le_bytes());
    bytes.extend_from_slice(&0_u16.to_le_bytes());
    bytes
}

pub fn segment(started_unix_us: f64, first_index: u64, setup: &SegmentSetup) -> Vec<u8> {
    let json = SegmentJson {
        started_unix_us,
        first_index,
        setup,
    };
    // Numbers, strings and booleans only: serialising them cannot fail.
    record(SEGMENT, &serde_json::to_vec(&json).unwrap_or_default())
}

/// Channel-major codes of the enabled channels. A chunk without samples gives `None`.
pub fn samples(first_index: u64, frame: &Frame) -> Option<Vec<u8>> {
    let present: Vec<&[u8]> = frame
        .channels
        .iter()
        .flatten()
        .map(|data| data.codes.as_slice())
        .collect();
    let count = present.first().map_or(0, |codes| codes.len());
    if count == 0 {
        return None;
    }
    let mut payload = Vec::with_capacity(12 + count * present.len());
    payload.extend_from_slice(&first_index.to_le_bytes());
    payload.extend_from_slice(&u32::try_from(count).unwrap_or(u32::MAX).to_le_bytes());
    for codes in present {
        payload.extend_from_slice(codes);
    }
    Some(record(SAMPLES, &payload))
}

pub fn sample_count(frame: &Frame) -> u64 {
    frame
        .channels
        .iter()
        .flatten()
        .map(|data| data.codes.len() as u64)
        .next()
        .unwrap_or(0)
}

pub fn gap(after: u64, missed: u64, estimated: bool) -> Vec<u8> {
    let mut payload = Vec::with_capacity(17);
    payload.extend_from_slice(&after.to_le_bytes());
    payload.extend_from_slice(&missed.to_le_bytes());
    payload.push(u8::from(estimated));
    record(GAP, &payload)
}

pub fn end(reason: EndReason) -> Vec<u8> {
    record(END, &[reason.code()])
}

fn record(kind: u8, payload: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(5 + payload.len());
    bytes.push(kind);
    bytes.extend_from_slice(&u32::try_from(payload.len()).unwrap_or(u32::MAX).to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes
}
