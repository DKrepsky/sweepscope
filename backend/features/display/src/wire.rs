//! Wire layout of ADR 0011.
use sweepscope_kernel::contracts::{ChannelData, DerivedTrace, Frame, FrameContext, TriggerOutcome};

use crate::convert::{scale_derived, scale_window, Window};
use crate::messages::InterpolationMode;
use crate::mode::upsample_factor;

const MAGIC: [u8; 4] = *b"SSF1";
const HEADER_LEN: usize = 48;
const MATH_CHANNEL: usize = 4;
const SENT_AT: usize = 16;
const FLAG_TRIGGERED: u8 = 0b01;
const FLAG_AUTO: u8 = 0b10;

/// ADR 0011 layout with the original window (what `Linear` and `Dots` send); the shell overwrites bytes 16..24 with the send time.
pub fn encode(frame: &Frame, context: &FrameContext, sequence: u32, produced_us: f64) -> Vec<u8> {
    encode_in_mode(frame, context, None, sequence, produced_us, InterpolationMode::Linear)
}

/// As `encode`; `Sinc` upsamples the window (ADR 0015 item 3) and scales interval and trigger index to match.
pub fn encode_in_mode(
    frame: &Frame,
    context: &FrameContext,
    derived: Option<&DerivedTrace>,
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

    let math = derived.filter(|trace| trace.volts.len() == record_len);
    let channel_count = present.len() + usize::from(math.is_some());
    let mut bytes = Vec::with_capacity(HEADER_LEN + 4 * points * channel_count);
    bytes.extend_from_slice(&MAGIC);
    bytes.extend_from_slice(&sequence.to_le_bytes());
    bytes.extend_from_slice(&produced_us.to_le_bytes());
    bytes.extend_from_slice(&0f64.to_le_bytes());
    let math_bit = if math.is_some() { 1 << MATH_CHANNEL } else { 0 };
    bytes.push(present.iter().fold(math_bit, |mask, (index, _)| mask | 1 << index));
    bytes.push(flags(frame.trigger));
    bytes.extend_from_slice(&[0, 0]);
    bytes.extend_from_slice(&(points as u32).to_le_bytes());
    bytes.extend_from_slice(&(frame.sample_interval_s / factor as f64).to_le_bytes());
    bytes.extend_from_slice(&scaled_trigger_index(window, frame.trigger, factor).to_le_bytes());
    bytes.extend_from_slice(&(context.setup_revision as u32).to_le_bytes());
    for (index, data) in present {
        let values = channel_values(data, frame.counts_per_div, window, context.invert[index], factor);
        push_values(&mut bytes, &values);
    }
    if let Some(trace) = math {
        push_values(&mut bytes, &crop_or_upsample(&scale_derived(trace), window, factor));
    }
    bytes
}

fn push_values(bytes: &mut Vec<u8>, values: &[f32]) {
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
}

fn crop_or_upsample(record: &[f32], window: Window, factor: usize) -> Vec<f32> {
    if factor == 1 {
        return record[window.start..window.start + window.len].to_vec();
    }
    interpolation::upsample_window(record, window.start, window.len, factor)
}

/// Upsampling scales the whole record first so the sinc reads real neighbours outside the window.
fn channel_values(data: &ChannelData, counts_per_div: f64, window: Window, invert: bool, factor: usize) -> Vec<f32> {
    if factor == 1 {
        return scale_window(data, counts_per_div, window, invert);
    }
    let whole = Window::within(data.codes.len(), 0, data.codes.len());
    let record = scale_window(data, counts_per_div, whole, invert);
    crop_or_upsample(&record, window, factor)
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
