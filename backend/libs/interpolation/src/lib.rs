//! Band-limited (sin(x)/x) upsampling of a window of a sample record (ADR 0015).
#![forbid(unsafe_code)]

/// Returns `(len - 1) * factor + 1` samples; `output[k * factor] == record[start + k]`.
/// Needs `factor >= 1` and `len >= 2`; reads the record outside the window to avoid edge ringing.
pub fn upsample_window(record: &[f32], start: usize, len: usize, factor: usize) -> Vec<f32> {
    let _ = (record, start, len, factor);
    unimplemented!("upsample_window: implemented by work package WP2")
}
