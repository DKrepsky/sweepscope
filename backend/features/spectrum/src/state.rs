//! The slice text: readouts of the record, the axes and whether the view can be used.
use engineering_format::Quantity;
use sweepscope_kernel::contracts::{Availability, FormattedValue};

use crate::analysis::Record;
use crate::axis::{ticks_x, Axis};
use crate::messages::{SpectrumScale, SpectrumState, SpectrumWindow};

const ROLL_REASON: &str = "FFT is not available in roll mode: it needs a full record.";

#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub active: bool,
    pub window: SpectrumWindow,
    pub scale: SpectrumScale,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            active: false,
            window: SpectrumWindow::Hann,
            scale: SpectrumScale::Dbv,
        }
    }
}

/// Revision 0; the slice numbers it.
pub fn build(settings: Settings, record: Option<Record>, axis: Axis, rolling: bool) -> SpectrumState {
    SpectrumState {
        revision: 0,
        active: settings.active,
        window: settings.window,
        scale: settings.scale,
        bin_spacing: shown(record.map(|r| r.bin_hz()), Quantity::Frequency),
        span: shown(record.map(|r| r.sample_rate_hz / 2.0), Quantity::Hertz),
        sample_rate: shown(record.map(|r| r.sample_rate_hz), Quantity::SamplesPerSecond),
        record_points: shown(record.map(|r| r.points as f64), Quantity::Points),
        top_label: axis.top_label(),
        ticks_x: ticks_x(record.map_or(0.0, |r| r.sample_rate_hz / 2.0)),
        ticks_y: axis.ticks_y(),
        availability: Availability {
            enabled: !rolling,
            reason: rolling.then(|| ROLL_REASON.to_owned()),
        },
    }
}

fn shown(value: Option<f64>, quantity: Quantity) -> FormattedValue {
    let formatted = engineering_format::format(value.unwrap_or(f64::NAN), quantity);
    FormattedValue {
        value: value.unwrap_or(0.0),
        text: formatted.text,
        unit: formatted.unit,
    }
}
