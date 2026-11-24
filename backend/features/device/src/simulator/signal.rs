//! Test signals of the simulated scope: clean waveforms plus deterministic seeded noise.
//!
//! Every waveform is a pure function of the simulated time, so frames stay continuous across captures.
//! Noise is a hash of the time and a fixed seed (own xorshift, no `rand`): the same instant always
//! yields the same noise, which keeps tests repeatable.

use std::f64::consts::TAU;

pub const FREQUENCY_HZ: f64 = 1_000.0;
/// Peak amplitude of `Sine`, `Square` and `Triangle` (they swing symmetrically around 0 V).
pub const AMPLITUDE_V: f64 = 1.0;
/// The calibrator output: 0 V low, 1.91 V high (driver measurements).
pub const CALIBRATOR_HIGH_V: f64 = 1.91;
/// 10 to 90 % rise and fall time of the calibrator edges (driver measurements).
pub const CALIBRATOR_EDGE_S: f64 = 250e-9;
/// Duration of the linear ramp whose 10 to 90 % part lasts `CALIBRATOR_EDGE_S`.
const CALIBRATOR_RAMP_S: f64 = CALIBRATOR_EDGE_S / 0.8;
pub const NOISE_AMPLITUDE_V: f64 = 0.002;
const NOISE_SEED: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignalKind {
    Sine,
    Square,
    Triangle,
    Calibrator,
    /// Pulses of `width_s` between the centres of their edges, one per period, starting at `t = 0`.
    /// A negative `amplitude_v` in the spec turns them into negative pulses (an inverted train).
    PulseTrain {
        width_s: f64,
    },
}

/// A waveform with its frequency, amplitude and DC offset. `new` gives the kind's usual values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SignalSpec {
    pub kind: SignalKind,
    pub frequency_hz: f64,
    /// Peak value of the waveform; for `Calibrator` and `PulseTrain` the height above the baseline.
    pub amplitude_v: f64,
    pub offset_v: f64,
}

impl SignalSpec {
    pub fn new(kind: SignalKind) -> Self {
        let amplitude_v = match kind {
            SignalKind::Calibrator => CALIBRATOR_HIGH_V,
            _ => AMPLITUDE_V,
        };
        Self {
            kind,
            frequency_hz: FREQUENCY_HZ,
            amplitude_v,
            offset_v: 0.0,
        }
    }

    pub fn with_frequency(mut self, frequency_hz: f64) -> Self {
        self.frequency_hz = frequency_hz;
        self
    }

    pub fn with_amplitude(mut self, amplitude_v: f64) -> Self {
        self.amplitude_v = amplitude_v;
        self
    }

    pub fn with_offset(mut self, offset_v: f64) -> Self {
        self.offset_v = offset_v;
        self
    }

    pub fn period_s(&self) -> f64 {
        1.0 / self.frequency_hz
    }
}

impl From<SignalKind> for SignalSpec {
    fn from(kind: SignalKind) -> Self {
        Self::new(kind)
    }
}

/// Voltage at simulated time `t` (seconds), noise included.
pub fn sample(spec: &SignalSpec, t: f64) -> f64 {
    clean(spec, t) + noise(t)
}

/// Voltage without noise. All waveforms start a period at `t = 0` going up (or high).
pub fn clean(spec: &SignalSpec, t: f64) -> f64 {
    let period = spec.period_s();
    let phase = (t / period).rem_euclid(1.0);
    let unit = match spec.kind {
        SignalKind::Sine => (TAU * phase).sin(),
        SignalKind::Square => square_wave(phase),
        SignalKind::Triangle => triangle_wave(phase),
        SignalKind::Calibrator => trapezoid_level(phase * period, period, period / 2.0),
        SignalKind::PulseTrain { width_s } => trapezoid_level(phase * period, period, width_s),
    };
    spec.offset_v + spec.amplitude_v * unit
}

fn square_wave(phase: f64) -> f64 {
    if phase < 0.5 {
        1.0
    } else {
        -1.0
    }
}

/// Trapezoid in 0..=1 with ramps centred on `t = 0` (rising) and on `width` (falling).
///
/// A width too short for the ramps or longer than the period minus them is held to what fits.
fn trapezoid_level(t: f64, period: f64, width: f64) -> f64 {
    let half_ramp = CALIBRATOR_RAMP_S / 2.0;
    let width = width.clamp(CALIBRATOR_RAMP_S, (period - CALIBRATOR_RAMP_S).max(CALIBRATOR_RAMP_S));
    if t < half_ramp {
        0.5 + t / CALIBRATOR_RAMP_S
    } else if t < width - half_ramp {
        1.0
    } else if t < width + half_ramp {
        0.5 - (t - width) / CALIBRATOR_RAMP_S
    } else if t < period - half_ramp {
        0.0
    } else {
        0.5 - (period - t) / CALIBRATOR_RAMP_S
    }
}

/// Starts at 0, peaks at a quarter period, crosses 0 going down at half, bottoms at three quarters.
fn triangle_wave(phase: f64) -> f64 {
    if phase < 0.25 {
        4.0 * phase
    } else if phase < 0.75 {
        2.0 - 4.0 * phase
    } else {
        4.0 * phase - 4.0
    }
}

fn noise(t: f64) -> f64 {
    let unit = xorshift64(t.to_bits() ^ NOISE_SEED) as f64 / u64::MAX as f64;
    NOISE_AMPLITUDE_V * (2.0 * unit - 1.0)
}

fn xorshift64(seed: u64) -> u64 {
    let mut x = seed.max(1);
    for _ in 0..3 {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    const QUARTER: f64 = 0.25 / FREQUENCY_HZ;

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn given_a_sine_when_sampled_at_quarter_periods_then_it_peaks_and_crosses_zero() {
        assert!(near(clean(&SignalKind::Sine.into(), 0.0), 0.0));
        assert!(near(clean(&SignalKind::Sine.into(), QUARTER), AMPLITUDE_V));
        assert!(near(clean(&SignalKind::Sine.into(), 3.0 * QUARTER), -AMPLITUDE_V));
    }

    #[test]
    fn given_a_square_when_sampled_in_each_half_period_then_it_is_high_then_low() {
        assert_eq!(clean(&SignalKind::Square.into(), QUARTER), AMPLITUDE_V);
        assert_eq!(clean(&SignalKind::Square.into(), 3.0 * QUARTER), -AMPLITUDE_V);
    }

    #[test]
    fn given_a_triangle_when_sampled_at_quarter_periods_then_it_follows_the_ramps() {
        assert!(near(clean(&SignalKind::Triangle.into(), 0.0), 0.0));
        assert!(near(clean(&SignalKind::Triangle.into(), QUARTER), AMPLITUDE_V));
        assert!(near(clean(&SignalKind::Triangle.into(), 2.0 * QUARTER), 0.0));
        assert!(near(clean(&SignalKind::Triangle.into(), 3.0 * QUARTER), -AMPLITUDE_V));
    }

    #[test]
    fn given_the_calibrator_when_sampled_then_it_is_a_1_khz_square_between_0_and_1_91_volts() {
        assert_eq!(clean(&SignalKind::Calibrator.into(), QUARTER), 1.91);
        assert_eq!(clean(&SignalKind::Calibrator.into(), 3.0 * QUARTER), 0.0);
        assert!(near(
            clean(&SignalKind::Calibrator.into(), QUARTER + 1.0 / FREQUENCY_HZ),
            1.91
        ));
    }

    #[test]
    fn given_the_calibrator_when_sampled_at_1_ns_then_the_10_to_90_percent_rise_is_250_ns_within_1_percent() {
        let level = |i: i64| clean(&SignalKind::Calibrator.into(), i as f64 * 1e-9);
        let first_at = |fraction: f64| (-500..500).find(|i| level(*i) >= fraction * CALIBRATOR_HIGH_V).unwrap();

        let rise_ns = (first_at(0.9) - first_at(0.1)) as f64;

        assert!((rise_ns - 250.0).abs() < 2.5, "{rise_ns}");
    }

    #[test]
    fn given_the_calibrator_when_sampled_at_1_ns_then_the_10_to_90_percent_fall_is_250_ns_within_1_percent() {
        let half = 0.5 / FREQUENCY_HZ;
        let level = |i: i64| clean(&SignalKind::Calibrator.into(), half + i as f64 * 1e-9);
        let first_below = |fraction: f64| (-500..500).find(|i| level(*i) <= fraction * CALIBRATOR_HIGH_V).unwrap();

        let fall_ns = (first_below(0.1) - first_below(0.9)) as f64;

        assert!((fall_ns - 250.0).abs() < 2.5, "{fall_ns}");
    }

    #[test]
    fn given_the_calibrator_when_sampled_at_each_edge_centre_then_it_is_at_half_level() {
        assert!(near(
            clean(&SignalKind::Calibrator.into(), 0.0),
            CALIBRATOR_HIGH_V / 2.0
        ));
        assert!(near(
            clean(&SignalKind::Calibrator.into(), 0.5 / FREQUENCY_HZ),
            CALIBRATOR_HIGH_V / 2.0
        ));
    }

    fn pulse_train(width_s: f64) -> SignalSpec {
        SignalSpec::new(SignalKind::PulseTrain { width_s })
    }

    #[test]
    fn given_a_pulse_train_when_sampled_then_it_is_high_for_the_width_and_low_for_the_rest_of_the_period() {
        let spec = pulse_train(200e-6);

        assert_eq!(clean(&spec, 100e-6), AMPLITUDE_V);
        assert_eq!(clean(&spec, 500e-6), 0.0);
        assert_eq!(clean(&spec, 1.1e-3 + 50e-6), AMPLITUDE_V);
    }

    #[test]
    fn given_an_inverted_pulse_train_when_sampled_then_the_pulse_goes_below_the_baseline() {
        let spec = pulse_train(200e-6).with_amplitude(-1.0);

        assert_eq!(clean(&spec, 100e-6), -1.0);
        assert_eq!(clean(&spec, 500e-6), 0.0);
    }

    #[test]
    fn given_a_frequency_amplitude_and_offset_when_sampled_then_the_waveform_follows_them() {
        let spec = SignalSpec::new(SignalKind::Square)
            .with_frequency(100.0)
            .with_amplitude(2.0)
            .with_offset(0.5);

        assert_eq!(clean(&spec, 0.0025), 2.5);
        assert_eq!(clean(&spec, 0.0075), -1.5);
        assert_eq!(clean(&spec, 0.0125), 2.5);
    }

    #[test]
    fn given_a_pulse_wider_than_the_period_when_sampled_then_the_width_is_held_to_what_fits() {
        let spec = pulse_train(5e-3);

        assert_eq!(clean(&spec, 0.5e-3), AMPLITUDE_V);
        assert!(clean(&spec, 0.99995e-3) < AMPLITUDE_V);
    }

    #[test]
    fn given_negative_time_when_sampled_then_the_waveform_stays_periodic() {
        assert!(near(clean(&SignalKind::Sine.into(), -3.0 * QUARTER), AMPLITUDE_V));
    }

    #[test]
    fn given_the_same_instant_when_sampled_twice_then_noise_is_identical() {
        assert_eq!(
            sample(&SignalKind::Sine.into(), 0.00123),
            sample(&SignalKind::Sine.into(), 0.00123)
        );
    }

    #[test]
    fn given_many_instants_when_sampled_then_noise_stays_within_its_amplitude_and_varies() {
        let noises: Vec<f64> = (0..1000)
            .map(|i| {
                sample(&SignalKind::Calibrator.into(), 0.0001 * i as f64)
                    - clean(&SignalKind::Calibrator.into(), 0.0001 * i as f64)
            })
            .collect();
        assert!(noises.iter().all(|n| n.abs() <= NOISE_AMPLITUDE_V));
        assert!(noises.iter().any(|n| *n > 0.0) && noises.iter().any(|n| *n < 0.0));
    }

    #[test]
    fn given_a_zero_seed_when_the_generator_runs_then_it_does_not_get_stuck_at_zero() {
        assert_ne!(xorshift64(0), 0);
    }
}
