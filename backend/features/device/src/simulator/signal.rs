//! Test signals of the simulated scope: clean waveforms plus deterministic seeded noise.
//!
//! Every waveform is a pure function of the simulated time, so frames stay continuous across captures.
//! Noise is a hash of the time and a fixed seed (own xorshift, no `rand`): the same instant always
//! yields the same noise, which keeps tests repeatable.

use std::f64::consts::TAU;

pub const FREQUENCY_HZ: f64 = 1_000.0;
/// Peak amplitude of `Sine`, `Square` and `Triangle` (they swing symmetrically around 0 V).
pub const AMPLITUDE_V: f64 = 1.0;
/// The calibrator output: 0 V low, 5 V high.
pub const CALIBRATOR_HIGH_V: f64 = 5.0;
pub const NOISE_AMPLITUDE_V: f64 = 0.002;
const NOISE_SEED: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalKind {
    Sine,
    Square,
    Triangle,
    Calibrator,
}

/// Voltage at simulated time `t` (seconds), noise included.
pub fn sample(kind: SignalKind, t: f64) -> f64 {
    clean(kind, t) + noise(t)
}

/// Voltage without noise. All waveforms start a period at `t = 0` going up (or high).
pub fn clean(kind: SignalKind, t: f64) -> f64 {
    let phase = (t * FREQUENCY_HZ).rem_euclid(1.0);
    match kind {
        SignalKind::Sine => AMPLITUDE_V * (TAU * phase).sin(),
        SignalKind::Square => AMPLITUDE_V * square_wave(phase),
        SignalKind::Triangle => AMPLITUDE_V * triangle_wave(phase),
        SignalKind::Calibrator => CALIBRATOR_HIGH_V * (square_wave(phase) + 1.0) / 2.0,
    }
}

fn square_wave(phase: f64) -> f64 {
    if phase < 0.5 {
        1.0
    } else {
        -1.0
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
        assert!(near(clean(SignalKind::Sine, 0.0), 0.0));
        assert!(near(clean(SignalKind::Sine, QUARTER), AMPLITUDE_V));
        assert!(near(clean(SignalKind::Sine, 3.0 * QUARTER), -AMPLITUDE_V));
    }

    #[test]
    fn given_a_square_when_sampled_in_each_half_period_then_it_is_high_then_low() {
        assert_eq!(clean(SignalKind::Square, QUARTER), AMPLITUDE_V);
        assert_eq!(clean(SignalKind::Square, 3.0 * QUARTER), -AMPLITUDE_V);
    }

    #[test]
    fn given_a_triangle_when_sampled_at_quarter_periods_then_it_follows_the_ramps() {
        assert!(near(clean(SignalKind::Triangle, 0.0), 0.0));
        assert!(near(clean(SignalKind::Triangle, QUARTER), AMPLITUDE_V));
        assert!(near(clean(SignalKind::Triangle, 2.0 * QUARTER), 0.0));
        assert!(near(clean(SignalKind::Triangle, 3.0 * QUARTER), -AMPLITUDE_V));
    }

    #[test]
    fn given_the_calibrator_when_sampled_then_it_is_a_1_khz_square_between_0_and_5_volts() {
        assert_eq!(clean(SignalKind::Calibrator, QUARTER), 5.0);
        assert_eq!(clean(SignalKind::Calibrator, 3.0 * QUARTER), 0.0);
        assert_eq!(clean(SignalKind::Calibrator, QUARTER + 1.0 / FREQUENCY_HZ), 5.0);
    }

    #[test]
    fn given_negative_time_when_sampled_then_the_waveform_stays_periodic() {
        assert!(near(clean(SignalKind::Sine, -3.0 * QUARTER), AMPLITUDE_V));
    }

    #[test]
    fn given_the_same_instant_when_sampled_twice_then_noise_is_identical() {
        assert_eq!(sample(SignalKind::Sine, 0.00123), sample(SignalKind::Sine, 0.00123));
    }

    #[test]
    fn given_many_instants_when_sampled_then_noise_stays_within_its_amplitude_and_varies() {
        let noises: Vec<f64> = (0..1000)
            .map(|i| {
                sample(SignalKind::Calibrator, 0.0001 * i as f64) - clean(SignalKind::Calibrator, 0.0001 * i as f64)
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
