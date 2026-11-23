use sweepscope_kernel::KernelError;

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

/// Public so tests and later offline mode can reload an export.
pub fn decode_raw(bytes: &[u8]) -> Result<RawCapture, KernelError> {
    let _ = bytes;
    Err(KernelError::fatal("decode_raw: implemented by work package WP7"))
}
