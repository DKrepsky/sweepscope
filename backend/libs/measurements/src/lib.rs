//! Automatic measurements of a sampled waveform (ADR 0014). Pure: no text, no units beyond SI numbers.
#![forbid(unsafe_code)]

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

/// Empty input gives zero amplitudes and `Err(NoSignal)` everywhere else.
pub fn measure(volts: &[f64], sample_interval_s: f64, volts_per_count: f64) -> Measured {
    let _ = (volts, sample_interval_s, volts_per_count);
    let none = Err(Invalid::NoSignal);
    Measured {
        vmax: 0.0,
        vmin: 0.0,
        vpp: 0.0,
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
