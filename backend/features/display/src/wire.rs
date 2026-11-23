//! Wire layout of ADR 0011.
use sweepscope_kernel::contracts::{Frame, FrameContext, TriggerOutcome};

use crate::convert::{scale_window, Window};

const MAGIC: [u8; 4] = *b"SSF1";
const HEADER_LEN: usize = 48;
const SENT_AT: usize = 16;
const FLAG_TRIGGERED: u8 = 0b01;
const FLAG_AUTO: u8 = 0b10;

/// ADR 0011 layout; the shell overwrites bytes 16..24 with the send time.
pub fn encode(frame: &Frame, context: &FrameContext, sequence: u32, produced_us: f64) -> Vec<u8> {
    let present: Vec<(usize, &_)> = frame
        .channels
        .iter()
        .enumerate()
        .filter_map(|(index, data)| data.as_ref().map(|data| (index, data)))
        .collect();
    let record_len = present.first().map_or(0, |(_, data)| data.codes.len());
    let window = Window::within(record_len, context.window_start, context.window_len);

    let mut bytes = Vec::with_capacity(HEADER_LEN + 4 * window.len * present.len());
    bytes.extend_from_slice(&MAGIC);
    bytes.extend_from_slice(&sequence.to_le_bytes());
    bytes.extend_from_slice(&produced_us.to_le_bytes());
    bytes.extend_from_slice(&0f64.to_le_bytes());
    bytes.push(present.iter().fold(0, |mask, (index, _)| mask | 1 << index));
    bytes.push(flags(frame.trigger));
    bytes.extend_from_slice(&[0, 0]);
    bytes.extend_from_slice(&(window.len as u32).to_le_bytes());
    bytes.extend_from_slice(&frame.sample_interval_s.to_le_bytes());
    bytes.extend_from_slice(&window.trigger_index(frame.trigger).to_le_bytes());
    bytes.extend_from_slice(&(context.setup_revision as u32).to_le_bytes());
    for (index, data) in present {
        for value in scale_window(data, frame.counts_per_div, window, context.invert[index]) {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    bytes
}

pub fn stamp_sent(bytes: &mut [u8], sent_us: f64) {
    if let Some(field) = bytes.get_mut(SENT_AT..SENT_AT + 8) {
        field.copy_from_slice(&sent_us.to_le_bytes());
    }
}

fn flags(trigger: TriggerOutcome) -> u8 {
    match trigger {
        TriggerOutcome::Triggered { .. } => FLAG_TRIGGERED,
        TriggerOutcome::Auto => FLAG_AUTO,
        TriggerOutcome::None => 0,
    }
}
