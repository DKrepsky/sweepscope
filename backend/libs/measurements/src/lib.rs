//! Automatic measurements of a sampled waveform (ADR 0014). Pure: no text, no units beyond SI numbers.
#![forbid(unsafe_code)]

mod crossings;
mod edges;
mod stats;

use crossings::Crossing;

const MIN_SWING_COUNTS: f64 = 10.0;
const HYSTERESIS_FRACTION: f64 = 0.1;

/// Why a quantity has no value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Invalid {
    NoSignal,
    FewerThanOnePeriod,
    NoCompleteEdge,
    EdgeTooFast,
    NoCompletePulse,
}

/// SI values: volts and seconds; `frequency` in hertz; `duty_cycle` in percent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measured {
    pub vmax: f64,
    pub vmin: f64,
    pub vpp: f64,
    pub vmean: Result<f64, Invalid>,
    pub vrms: Result<f64, Invalid>,
    pub frequency: Result<f64, Invalid>,
    pub period: Result<f64, Invalid>,
    pub positive_width: Result<f64, Invalid>,
    pub negative_width: Result<f64, Invalid>,
    pub duty_cycle: Result<f64, Invalid>,
    pub rise_time: Result<f64, Invalid>,
    pub fall_time: Result<f64, Invalid>,
}

impl Measured {
    fn all_invalid(vmax: f64, vmin: f64, reason: Invalid) -> Self {
        let none = Err(reason);
        Self {
            vmax,
            vmin,
            vpp: vmax - vmin,
            vmean: none,
            vrms: none,
            frequency: none,
            period: none,
            positive_width: none,
            negative_width: none,
            duty_cycle: none,
            rise_time: none,
            fall_time: none,
        }
    }
}

/// Empty input gives zero amplitudes and `Err(NoSignal)` everywhere else.
pub fn measure(volts: &[f64], sample_interval_s: f64, volts_per_count: f64) -> Measured {
    if volts.is_empty() {
        return Measured::all_invalid(0.0, 0.0, Invalid::NoSignal);
    }
    let (vmin, vmax) = volts
        .iter()
        .fold((f64::MAX, f64::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
    let vpp = vmax - vmin;
    if vpp < MIN_SWING_COUNTS * volts_per_count {
        let (mean, rms) = stats::mean_and_rms(volts);
        return Measured {
            vmean: Ok(mean),
            vrms: Ok(rms),
            ..Measured::all_invalid(vmax, vmin, Invalid::NoSignal)
        };
    }
    let mid = (vmin + vmax) / 2.0;
    let hysteresis = (HYSTERESIS_FRACTION * vpp).max(2.0 * volts_per_count);
    let crossings = crossings::find(volts, mid, hysteresis);
    let rising = crossings::rising_positions(&crossings);
    let edges = edges::measure(volts, &crossings, mid, sample_interval_s);
    let (vmean, vrms) = stats::mean_and_rms(whole_periods(volts, &rising));
    Measured {
        vmax,
        vmin,
        vpp,
        vmean: Ok(vmean),
        vrms: Ok(vrms),
        rise_time: edges.rise,
        fall_time: edges.fall,
        ..timing(&crossings, sample_interval_s)
    }
}

fn whole_periods<'a>(volts: &'a [f64], rising: &[f64]) -> &'a [f64] {
    match (rising.first(), rising.last()) {
        (Some(first), Some(last)) if rising.len() >= 2 => stats::between(volts, *first, *last),
        _ => volts,
    }
}

/// Period, frequency, widths and duty cycle; the other fields are placeholders for the caller to replace.
fn timing(crossings: &[Crossing], sample_interval_s: f64) -> Measured {
    let mut measured = Measured::all_invalid(0.0, 0.0, Invalid::FewerThanOnePeriod);
    if let Some(samples) = crossings::fitted_period(crossings) {
        let period = samples * sample_interval_s;
        measured.period = Ok(period);
        measured.frequency = Ok(1.0 / period);
    }
    let width = |starts_rising| {
        crossings::mean_pulse_width(crossings, starts_rising)
            .map(|samples| samples * sample_interval_s)
            .ok_or(Invalid::NoCompletePulse)
    };
    measured.positive_width = width(true);
    measured.negative_width = width(false);
    measured.duty_cycle = match (measured.positive_width, measured.negative_width) {
        (Ok(high), Ok(low)) => Ok(100.0 * high / (high + low)),
        _ => Err(Invalid::NoCompletePulse),
    };
    measured
}

/// True when any code sits on a converter rail (0 or 255).
pub fn is_clipped(codes: &[u8]) -> bool {
    codes.iter().any(|&code| code == u8::MIN || code == u8::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_a_code_on_a_rail_when_checked_then_the_record_is_clipped() {
        assert!(is_clipped(&[10, 255, 20]));
        assert!(is_clipped(&[0, 100]));
    }

    #[test]
    fn given_codes_inside_the_rails_when_checked_then_the_record_is_not_clipped() {
        assert!(!is_clipped(&[1, 128, 254]));
        assert!(!is_clipped(&[]));
    }

    #[test]
    fn given_empty_input_when_measured_then_every_result_is_no_signal() {
        let measured = measure(&[], 1e-6, 0.01);

        assert_eq!(measured.vmean, Err(Invalid::NoSignal));
        assert_eq!(measured.fall_time, Err(Invalid::NoSignal));
        assert_eq!(measured.vpp, 0.0);
    }
}
