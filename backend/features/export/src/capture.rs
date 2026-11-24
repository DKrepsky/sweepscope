//! The newest frame as the export formats see it, with what it needs to convert codes to volts.
use sweepscope_kernel::contracts::{FrameCaptured, TriggerOutcome};
use sweepscope_kernel::KernelError;

pub struct ChannelCapture {
    pub channel: u8,
    pub volts_per_div: f64,
    pub probe: f64,
    pub zero_code: f64,
    pub inverted: bool,
    pub calibrated: bool,
    pub codes: Vec<u8>,
}

pub struct Capture {
    pub timestamp_utc: String,
    pub sample_interval_s: f64,
    pub seconds_per_div: f64,
    pub counts_per_div: f64,
    pub trigger_index: Option<usize>,
    pub record_points: usize,
    /// Ascending channel order, enabled channels only.
    pub channels: Vec<ChannelCapture>,
}

impl Capture {
    /// Recoverable `No waveform to export` when the frame holds no samples.
    pub fn from_frame(captured: &FrameCaptured, timestamp_utc: String) -> Result<Self, KernelError> {
        let frame = &captured.frame;
        let context = &captured.context;
        let channels: Vec<ChannelCapture> = frame
            .channels
            .iter()
            .enumerate()
            .filter_map(|(index, data)| {
                let data = data.as_ref().filter(|data| !data.codes.is_empty())?;
                Some(ChannelCapture {
                    channel: index as u8 + 1,
                    volts_per_div: data.volts_per_div,
                    probe: context.probe[index],
                    zero_code: data.zero_code,
                    inverted: context.invert[index],
                    calibrated: context.calibrated[index],
                    codes: data.codes.clone(),
                })
            })
            .collect();
        let record_points = channels.first().map_or(0, |channel| channel.codes.len());
        if record_points == 0 || channels.iter().any(|channel| channel.codes.len() != record_points) {
            return Err(KernelError::recoverable("No waveform to export"));
        }
        let trigger_index = match frame.trigger {
            TriggerOutcome::Triggered { sample_index } => Some(sample_index),
            TriggerOutcome::Auto | TriggerOutcome::None => None,
        };
        Ok(Self {
            timestamp_utc,
            sample_interval_s: frame.sample_interval_s,
            seconds_per_div: context.seconds_per_div,
            counts_per_div: frame.counts_per_div,
            trigger_index,
            record_points,
            channels,
        })
    }
}
