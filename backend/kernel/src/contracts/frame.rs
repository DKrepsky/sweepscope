/// Raw 8-bit codes of one channel with what is needed to convert them to volts.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelData {
    pub codes: Vec<u8>,
    pub volts_per_div: f64,
    pub zero_code: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerOutcome {
    Triggered {
        sample_index: usize,
    },
    /// Free-run frame after the auto sweep gave up waiting.
    Auto,
    /// Roll chunks have no trigger.
    None,
}

/// One capture or one roll chunk. A roll chunk may hold no samples.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    /// `None` for a disabled channel.
    pub channels: [Option<ChannelData>; 4],
    pub counts_per_div: f64,
    pub sample_interval_s: f64,
    pub trigger: TriggerOutcome,
}
