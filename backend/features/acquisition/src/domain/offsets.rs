//! Zero offsets of the connected unit applied to what the device delivers (ADR 0021 item 4).
use std::sync::Arc;

use sweepscope_kernel::contracts::{Capabilities, Frame, ZeroOffsets};

use super::model::CHANNELS;

/// Two table entries closer than this (relative) are the same range.
const RANGE_MATCH: f64 = 1e-6;

/// Input-referred offset of a channel on a range; zero where none was measured.
pub fn volts_at(offsets: &ZeroOffsets, channel: usize, range_index: usize) -> f64 {
    offsets
        .volts
        .get(channel)
        .and_then(|per_range| per_range.get(range_index))
        .copied()
        .unwrap_or(0.0)
}

pub struct Corrected {
    pub frame: Arc<Frame>,
    pub calibrated: [bool; CHANNELS],
}

/// `zero_code` moves by the offset in codes, so every consumer that converts with it is corrected at once. The frame
/// is copied only when offsets are in force.
pub fn correct(caps: &Capabilities, offsets: Option<&ZeroOffsets>, frame: &Arc<Frame>) -> Corrected {
    let mut calibrated = [false; CHANNELS];
    let Some(offsets) = offsets else {
        return Corrected {
            frame: Arc::clone(frame),
            calibrated,
        };
    };
    let mut corrected = Frame::clone(frame);
    for (number, slot) in corrected.channels.iter_mut().enumerate() {
        let Some(data) = slot else { continue };
        let Some(index) = range_index(caps, data.volts_per_div) else {
            continue;
        };
        let volts = volts_at(offsets, number, index);
        data.zero_code += volts / data.volts_per_div * frame.counts_per_div;
        calibrated[number] = offsets.volts[number].get(index).is_some();
    }
    Corrected {
        frame: Arc::new(corrected),
        calibrated,
    }
}

fn range_index(caps: &Capabilities, volts_per_div: f64) -> Option<usize> {
    caps.vertical_ranges
        .iter()
        .position(|r| (r.volts_per_div - volts_per_div).abs() <= RANGE_MATCH * volts_per_div.abs())
}
