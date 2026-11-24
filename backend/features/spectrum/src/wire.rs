//! Wire layout `SSS1` of ADR 0018 item 4 (little endian).
use crate::messages::SpectrumWindow;

const MAGIC: [u8; 4] = *b"SSS1";
const SENT_AT: usize = 16;
const FLAG_LINEAR: u8 = 0b1;
const NO_PEAK: u32 = u32::MAX;

/// Slot 0 to 3 are CH1 to CH4, slot 4 is the math channel (mask bit 4).
pub const MATH_SLOT: u8 = 4;

pub struct FrameHeader {
    pub linear: bool,
    pub window: SpectrumWindow,
    pub bin_hz: f64,
    pub top: f32,
    pub units_per_div: f32,
    pub setup_revision: u32,
}

pub struct ChannelSpectrum {
    pub slot: u8,
    pub peak_bin: Option<u32>,
    pub levels: Vec<f32>,
    /// Distance of each level from the top edge, in divisions, so the graph draws without computing.
    pub divisions: Vec<f32>,
}

/// Channels must be given in ascending slot order and all have `levels.len()` bins; the shell overwrites bytes 16..24 with the send time.
pub fn encode(header: &FrameHeader, channels: &[ChannelSpectrum], sequence: u32, produced_us: f64) -> Vec<u8> {
    let bins = channels.first().map_or(0, |channel| channel.levels.len());
    let mut bytes = Vec::with_capacity(56 + channels.len() * (4 + 8 * bins));
    bytes.extend_from_slice(&MAGIC);
    bytes.extend_from_slice(&sequence.to_le_bytes());
    bytes.extend_from_slice(&produced_us.to_le_bytes());
    bytes.extend_from_slice(&0f64.to_le_bytes());
    bytes.push(channels.iter().fold(0, |mask, channel| mask | 1 << channel.slot));
    bytes.push(if header.linear { FLAG_LINEAR } else { 0 });
    bytes.push(window_code(header.window));
    bytes.push(0);
    bytes.extend_from_slice(&(bins as u32).to_le_bytes());
    bytes.extend_from_slice(&header.bin_hz.to_le_bytes());
    bytes.extend_from_slice(&header.top.to_le_bytes());
    bytes.extend_from_slice(&header.units_per_div.to_le_bytes());
    bytes.extend_from_slice(&header.setup_revision.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    for channel in channels {
        bytes.extend_from_slice(&channel.peak_bin.unwrap_or(NO_PEAK).to_le_bytes());
    }
    for channel in channels {
        for value in channel.levels.iter().chain(&channel.divisions) {
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

fn window_code(window: SpectrumWindow) -> u8 {
    match window {
        SpectrumWindow::Rectangular => 0,
        SpectrumWindow::Hann => 1,
        SpectrumWindow::Hamming => 2,
        SpectrumWindow::Blackman => 3,
    }
}
