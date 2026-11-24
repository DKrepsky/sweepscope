//! QA characterisation of R2 (M2 spec section 6): worst-case frequency error of the simulator-like calibrator
//! (0 to 1.91 V, 1 kHz, 250 ns edges, 8-bit codes at 100 counts per division) across time bases and trigger phases.

use measurements::measure;

const RECORD: usize = 4096;
const COUNTS_PER_DIV: f64 = 100.0;
const ZERO_CODE: f64 = 30.0;
const PERIOD: f64 = 1e-3;
const RAMP: f64 = 250e-9 / 0.8;

fn level(t: f64) -> f64 {
    let phase = t.rem_euclid(PERIOD);
    let half = PERIOD / 2.0;
    let unit = if phase < RAMP {
        phase / RAMP
    } else if phase < half {
        1.0
    } else if phase < half + RAMP {
        1.0 - (phase - half) / RAMP
    } else {
        0.0
    };
    unit * 1.91
}

/// Worst relative frequency error over 200 phases; `noise_counts` is a deterministic pseudo noise amplitude.
fn worst_error(seconds_per_div: f64, noise_counts: f64) -> (f64, f64) {
    let interval = seconds_per_div * 16.4 / RECORD as f64;
    let (mut worst, mut sum, mut invalid) = (0.0_f64, 0.0_f64, 0_usize);
    for step in 0..200 {
        let offset = step as f64 * PERIOD / 200.0 * 1.0037;
        let volts: Vec<f64> = (0..RECORD)
            .map(|i| {
                let t = offset + i as f64 * interval;
                let noise = ((t * 1.0e9 * 0.618_033_988).sin() * 43_758.545).fract() * noise_counts;
                let code = (ZERO_CODE + level(t) * COUNTS_PER_DIV + noise).round();
                (code - ZERO_CODE) / COUNTS_PER_DIV
            })
            .collect();
        let measured = measure(&volts, interval, 1.0 / COUNTS_PER_DIV);
        let Ok(frequency) = measured.frequency else {
            invalid += 1;
            continue;
        };
        let error = (frequency - 1000.0).abs() / 1000.0;
        worst = worst.max(error);
        sum += error;
    }
    println!("  invalid phases: {invalid} of 200");
    (worst, sum / (200 - invalid).max(1) as f64)
}

fn assert_worst_within(name: &str, per_div: f64, noise: f64, bound: f64) {
    let (worst, mean) = worst_error(per_div, noise);
    println!(
        "R2 {name}/div noise {noise} counts: worst {:.4} %, mean {:.4} %",
        worst * 100.0,
        mean * 100.0
    );
    assert!(
        worst <= bound,
        "{name}: worst error {:.4} % exceeds {:.2} %",
        worst * 100.0,
        bound * 100.0
    );
}

#[test]
fn given_the_calibrator_at_500_us_and_slower_when_the_phase_sweeps_then_frequency_is_within_0_01_percent() {
    for (name, per_div) in [("500 us", 500e-6), ("1 ms", 1e-3), ("2 ms", 2e-3)] {
        for noise in [0.0, 0.4] {
            assert_worst_within(name, per_div, noise, 1e-4);
        }
    }
}

#[test]
fn given_the_calibrator_at_faster_time_bases_when_the_phase_sweeps_then_frequency_is_within_0_02_percent() {
    for (name, per_div) in [("100 us", 100e-6), ("200 us", 200e-6)] {
        for noise in [0.0, 0.4] {
            assert_worst_within(name, per_div, noise, 2e-4);
        }
    }
}
