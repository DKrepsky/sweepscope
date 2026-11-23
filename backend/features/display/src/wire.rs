//! Wire layout of ADR 0011.
use sweepscope_kernel::contracts::{ChannelData, Frame, FrameContext, TriggerOutcome};

use crate::convert::{scale_window, Window};
use crate::messages::InterpolationMode;
use crate::mode::upsample_factor;

const MAGIC: [u8; 4] = *b"SSF1";
const HEADER_LEN: usize = 48;
const SENT_AT: usize = 16;
const FLAG_TRIGGERED: u8 = 0b01;
const FLAG_AUTO: u8 = 0b10;

/// ADR 0011 layout with the original window (what `Linear` and `Dots` send); the shell overwrites bytes 16..24 with the send time.
pub fn encode(frame: &Frame, context: &FrameContext, sequence: u32, produced_us: f64) -> Vec<u8> {
    encode_in_mode(frame, context, sequence, produced_us, InterpolationMode::Linear)
}

/// As `encode`; `Sinc` upsamples the window (ADR 0015 item 3) and scales interval and trigger index to match.
pub fn encode_in_mode(
    frame: &Frame,
    context: &FrameContext,
    sequence: u32,
    produced_us: f64,
    mode: InterpolationMode,
) -> Vec<u8> {
    let present: Vec<(usize, &_)> = frame
        .channels
        .iter()
        .enumerate()
        .filter_map(|(index, data)| data.as_ref().map(|data| (index, data)))
        .collect();
    let record_len = present.first().map_or(0, |(_, data)| data.codes.len());
    let window = Window::within(record_len, context.window_start, context.window_len);
    let factor = match mode {
        InterpolationMode::Sinc if window.len >= 2 => upsample_factor(window.len),
        _ => 1,
    };
    let points = if window.len == 0 {
        0
    } else {
        (window.len - 1) * factor + 1
    };

    let mut bytes = Vec::with_capacity(HEADER_LEN + 4 * points * present.len());
    bytes.extend_from_slice(&MAGIC);
    bytes.extend_from_slice(&sequence.to_le_bytes());
    bytes.extend_from_slice(&produced_us.to_le_bytes());
    bytes.extend_from_slice(&0f64.to_le_bytes());
    bytes.push(present.iter().fold(0, |mask, (index, _)| mask | 1 << index));
    bytes.push(flags(frame.trigger));
    bytes.extend_from_slice(&[0, 0]);
    bytes.extend_from_slice(&(points as u32).to_le_bytes());
    bytes.extend_from_slice(&(frame.sample_interval_s / factor as f64).to_le_bytes());
    bytes.extend_from_slice(&scaled_trigger_index(window, frame.trigger, factor).to_le_bytes());
    bytes.extend_from_slice(&(context.setup_revision as u32).to_le_bytes());
    for (index, data) in present {
        let values = channel_values(data, frame.counts_per_div, window, context.invert[index], factor);
        for value in values {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    bytes
}

/// Upsampling scales the whole record first so the sinc reads real neighbours outside the window.
fn channel_values(data: &ChannelData, counts_per_div: f64, window: Window, invert: bool, factor: usize) -> Vec<f32> {
    if factor == 1 {
        return scale_window(data, counts_per_div, window, invert);
    }
    let whole = Window::within(data.codes.len(), 0, data.codes.len());
    let record = scale_window(data, counts_per_div, whole, invert);
    interpolation::upsample_window(&record, window.start, window.len, factor)
}

pub fn stamp_sent(bytes: &mut [u8], sent_us: f64) {
    if let Some(field) = bytes.get_mut(SENT_AT..SENT_AT + 8) {
        field.copy_from_slice(&sent_us.to_le_bytes());
    }
}

fn scaled_trigger_index(window: Window, trigger: TriggerOutcome, factor: usize) -> i32 {
    let index = window.trigger_index(trigger);
    if index < 0 {
        return -1;
    }
    i32::try_from(index as usize * factor).unwrap_or(-1)
}

fn flags(trigger: TriggerOutcome) -> u8 {
    match trigger {
        TriggerOutcome::Triggered { .. } => FLAG_TRIGGERED,
        TriggerOutcome::Auto => FLAG_AUTO,
        TriggerOutcome::None => 0,
    }
}
