//! One channel of one frame to its twelve measurements.
use engineering_format::{format, Quantity};
use measurements::{is_clipped, measure, Invalid as LibInvalid, Measured};
use sweepscope_kernel::contracts::{ChannelData, DerivedTrace, FormattedValue, Frame, FrameContext};

use crate::messages::{ChannelMeasurements, InvalidReason, Measurement, MeasurementId};
use crate::texts;

type Cell = Result<(f64, Quantity), InvalidReason>;

/// Column of the math trace.
const MATH_CHANNEL: u8 = 5;

/// Enabled channels of the frame, ascending, then the math trace as channel 5; `None` when a record holds no
/// samples (a roll chunk may be empty).
pub fn measure_frame(
    frame: &Frame,
    context: &FrameContext,
    derived: Option<&DerivedTrace>,
) -> Option<Vec<ChannelMeasurements>> {
    let mut channels = Vec::new();
    for (index, data) in frame.channels.iter().enumerate() {
        let Some(data) = data else { continue };
        if data.codes.is_empty() {
            return None;
        }
        channels.push(measure_channel(frame, context, index, data));
    }
    channels.extend(
        derived
            .filter(|d| !d.volts.is_empty())
            .map(|d| measure_derived(frame, d)),
    );
    Some(channels)
}

/// The math trace is already in volts; it has no converter rails, so it never clips and has no range flag.
fn measure_derived(frame: &Frame, derived: &DerivedTrace) -> ChannelMeasurements {
    let volts_per_count = derived.volts_per_div / frame.counts_per_div;
    let measured = measure(&derived.volts, frame.sample_interval_s, volts_per_count);
    ChannelMeasurements {
        channel: MATH_CHANNEL,
        clipped: false,
        measurements: cells(&measured, false),
    }
}

fn measure_channel(frame: &Frame, context: &FrameContext, index: usize, data: &ChannelData) -> ChannelMeasurements {
    let volts_per_count = data.volts_per_div * context.probe[index] / frame.counts_per_div;
    let volts: Vec<f64> = data
        .codes
        .iter()
        .map(|&code| (f64::from(code) - data.zero_code) * volts_per_count)
        .collect();
    let measured = measure(&volts, frame.sample_interval_s, volts_per_count);
    let clipped = is_clipped(&data.codes);
    ChannelMeasurements {
        channel: index as u8 + 1,
        clipped,
        measurements: cells(&measured, clipped),
    }
}

fn cells(m: &Measured, clipped: bool) -> Vec<Measurement> {
    use MeasurementId as Id;
    let timing = |result: Result<f64, LibInvalid>, quantity| result.map(|v| (v, quantity)).map_err(texts::reason_of);
    let amplitude = |result: Result<f64, LibInvalid>| {
        if clipped {
            Err(InvalidReason::Clipped)
        } else {
            timing(result, Quantity::Volts)
        }
    };
    let rows: [(Id, Cell); 12] = [
        (Id::Frequency, timing(m.frequency, Quantity::Frequency)),
        (Id::Period, timing(m.period, Quantity::Period)),
        (Id::Vpp, amplitude(Ok(m.vpp))),
        (Id::Vmax, amplitude(Ok(m.vmax))),
        (Id::Vmin, amplitude(Ok(m.vmin))),
        (Id::Vmean, amplitude(m.vmean)),
        (Id::Vrms, amplitude(m.vrms)),
        (Id::PositiveWidth, timing(m.positive_width, Quantity::Seconds)),
        (Id::NegativeWidth, timing(m.negative_width, Quantity::Seconds)),
        (Id::DutyCycle, timing(m.duty_cycle, Quantity::Percent)),
        (Id::RiseTime, edge(m.rise_time, clipped)),
        (Id::FallTime, edge(m.fall_time, clipped)),
    ];
    rows.into_iter().map(|(id, cell)| to_measurement(id, cell)).collect()
}

fn edge(result: Result<f64, LibInvalid>, clipped: bool) -> Cell {
    if clipped {
        return Err(InvalidReason::Clipped);
    }
    result.map(|v| (v, Quantity::Seconds)).map_err(texts::reason_of)
}

fn to_measurement(id: MeasurementId, cell: Cell) -> Measurement {
    let (value, invalid) = match cell {
        Ok((number, quantity)) => {
            let formatted = format(number, quantity);
            let value = FormattedValue {
                value: formatted.value,
                text: formatted.text,
                unit: formatted.unit,
            };
            (Some(value), None)
        }
        Err(reason) => (None, Some(texts::invalid(reason))),
    };
    Measurement {
        id,
        label: texts::label(id).to_owned(),
        value,
        invalid,
    }
}
