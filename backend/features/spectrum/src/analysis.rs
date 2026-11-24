//! One captured record to the spectra of its channels and of the math trace.
use fft::{peak_bin, spectrum, Spectrum, Window};
use sweepscope_kernel::contracts::{ChannelData, DerivedUnit, Frame, FrameCaptured, FrameContext};

use crate::messages::SpectrumWindow;
use crate::wire::MATH_SLOT;

pub struct Analysed {
    pub slot: u8,
    pub spectrum: Spectrum,
    pub peak_bin: Option<usize>,
}

pub struct Analysis {
    pub record: Record,
    pub channels: Vec<Analysed>,
}

/// What the readouts say about the record, known even while no spectrum is computed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Record {
    pub sample_rate_hz: f64,
    pub points: usize,
}

impl Record {
    pub fn bin_hz(&self) -> f64 {
        self.sample_rate_hz / self.points as f64
    }
}

pub fn record_of(frame: &Frame) -> Option<Record> {
    let points = frame.channels.iter().flatten().map(|data| data.codes.len()).next()?;
    let valid = frame.sample_interval_s.is_finite() && frame.sample_interval_s > 0.0 && points > 0;
    valid.then(|| Record {
        sample_rate_hz: 1.0 / frame.sample_interval_s,
        points,
    })
}

/// `None` when there is no channel or the record is not a power of two long (the transform refuses it).
pub fn analyse(captured: &FrameCaptured, window: SpectrumWindow) -> Option<Analysis> {
    let record = record_of(&captured.frame)?;
    let lib_window = lib_window(window);
    let traces = traces_of(captured);
    let channels: Vec<Analysed> = traces
        .into_iter()
        .filter_map(|(slot, volts)| {
            let spectrum = spectrum(&volts, lib_window, record.sample_rate_hz).ok()?;
            let peak_bin = peak_bin(&spectrum, lib_window);
            Some(Analysed {
                slot,
                spectrum,
                peak_bin,
            })
        })
        .collect();
    (!channels.is_empty()).then_some(Analysis { record, channels })
}

fn traces_of(captured: &FrameCaptured) -> Vec<(u8, Vec<f64>)> {
    let frame = &captured.frame;
    let mut traces: Vec<(u8, Vec<f64>)> = frame
        .channels
        .iter()
        .enumerate()
        .filter_map(|(index, data)| {
            let data = data.as_ref()?;
            Some((index as u8, volts(data, frame.counts_per_div, &captured.context, index)))
        })
        .collect();
    let math = captured
        .derived
        .as_ref()
        .filter(|derived| derived.unit == DerivedUnit::Volts);
    if let Some(derived) = math {
        traces.push((MATH_SLOT, derived.volts.clone()));
    }
    traces
}

fn volts(data: &ChannelData, counts_per_div: f64, context: &FrameContext, index: usize) -> Vec<f64> {
    let volts_per_count = data.volts_per_div * context.probe[index] / counts_per_div;
    data.codes
        .iter()
        .map(|&code| (f64::from(code) - data.zero_code) * volts_per_count)
        .collect()
}

fn lib_window(window: SpectrumWindow) -> Window {
    match window {
        SpectrumWindow::Rectangular => Window::Rectangular,
        SpectrumWindow::Hann => Window::Hann,
        SpectrumWindow::Hamming => Window::Hamming,
        SpectrumWindow::Blackman => Window::Blackman,
    }
}
