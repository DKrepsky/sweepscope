use serde::{Deserialize, Serialize};
use sweepscope_kernel::KernelError;

use crate::capture::Capture;

const MAGIC: &[u8; 4] = b"SSR1";
const VERSION: u32 = 1;
const FORMAT: &str = "sweepscope-raw";

/// One channel of a raw capture: the header entry plus its codes.
#[derive(Debug, Clone, PartialEq)]
pub struct RawChannel {
    pub channel: u8,
    pub volts_per_div: f64,
    pub probe: f64,
    pub zero_code: f64,
    pub inverted: bool,
    pub range_verified: bool,
    pub codes: Vec<u8>,
}

/// What an `SSR1` file holds (layout in `docs/formats/frame-raw-v1.md`).
#[derive(Debug, Clone, PartialEq)]
pub struct RawCapture {
    pub timestamp_utc: String,
    pub sample_interval_s: f64,
    pub seconds_per_div: f64,
    pub record_points: usize,
    pub counts_per_div: f64,
    pub trigger_index: Option<usize>,
    /// Ascending channel order.
    pub channels: Vec<RawChannel>,
}

#[derive(Serialize, Deserialize)]
struct HeaderChannel {
    channel: u8,
    volts_per_div: f64,
    probe: f64,
    zero_code: f64,
    inverted: bool,
    range_verified: bool,
}

#[derive(Serialize, Deserialize)]
struct Header {
    format: String,
    version: u32,
    timestamp_utc: String,
    sample_interval_s: f64,
    seconds_per_div: f64,
    record_points: usize,
    counts_per_div: f64,
    trigger_index: Option<usize>,
    channels: Vec<HeaderChannel>,
}

pub fn encode(capture: &Capture) -> Result<Vec<u8>, KernelError> {
    let header = Header {
        format: FORMAT.into(),
        version: VERSION,
        timestamp_utc: capture.timestamp_utc.clone(),
        sample_interval_s: capture.sample_interval_s,
        seconds_per_div: capture.seconds_per_div,
        record_points: capture.record_points,
        counts_per_div: capture.counts_per_div,
        trigger_index: capture.trigger_index,
        channels: capture
            .channels
            .iter()
            .map(|channel| HeaderChannel {
                channel: channel.channel,
                volts_per_div: channel.volts_per_div,
                probe: channel.probe,
                zero_code: channel.zero_code,
                inverted: channel.inverted,
                range_verified: channel.range_verified,
            })
            .collect(),
    };
    let json = serde_json::to_vec(&header).map_err(|e| KernelError::fatal(format!("cannot encode the header: {e}")))?;
    let length = u32::try_from(json.len()).map_err(|_| KernelError::fatal("raw header is too large"))?;
    let mut bytes = Vec::with_capacity(8 + json.len() + capture.record_points * capture.channels.len());
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&length.to_le_bytes());
    bytes.extend_from_slice(&json);
    for channel in &capture.channels {
        bytes.extend_from_slice(&channel.codes);
    }
    Ok(bytes)
}

fn invalid(reason: &str) -> KernelError {
    KernelError::recoverable(format!("not a SweepScope raw capture: {reason}"))
}

fn read_header(bytes: &[u8]) -> Result<(Header, &[u8]), KernelError> {
    let prefix = bytes.get(..8).ok_or_else(|| invalid("the file is too short"))?;
    if &prefix[..4] != MAGIC {
        return Err(invalid("the magic bytes are wrong"));
    }
    let length = u32::from_le_bytes([prefix[4], prefix[5], prefix[6], prefix[7]]) as usize;
    let json = bytes
        .get(8..8 + length)
        .ok_or_else(|| invalid("the header is cut short"))?;
    let header: Header =
        serde_json::from_slice(json).map_err(|e| invalid(&format!("the header is unreadable ({e})")))?;
    if header.format != FORMAT || header.version != VERSION {
        return Err(invalid("unsupported format or version"));
    }
    Ok((header, &bytes[8 + length..]))
}

/// Public so tests and later offline mode can reload an export.
pub fn decode_raw(bytes: &[u8]) -> Result<RawCapture, KernelError> {
    let (header, mut rest) = read_header(bytes)?;
    let mut channels = Vec::with_capacity(header.channels.len());
    for entry in header.channels {
        if rest.len() < header.record_points {
            return Err(invalid("the sample data is cut short"));
        }
        let (codes, tail) = rest.split_at(header.record_points);
        rest = tail;
        channels.push(RawChannel {
            channel: entry.channel,
            volts_per_div: entry.volts_per_div,
            probe: entry.probe,
            zero_code: entry.zero_code,
            inverted: entry.inverted,
            range_verified: entry.range_verified,
            codes: codes.to_vec(),
        });
    }
    if !rest.is_empty() {
        return Err(invalid("there are bytes after the sample data"));
    }
    Ok(RawCapture {
        timestamp_utc: header.timestamp_utc,
        sample_interval_s: header.sample_interval_s,
        seconds_per_div: header.seconds_per_div,
        record_points: header.record_points,
        counts_per_div: header.counts_per_div,
        trigger_index: header.trigger_index,
        channels,
    })
}
