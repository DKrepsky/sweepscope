use super::*;
use std::f64::consts::{SQRT_2, TAU};

const WINDOWS: [Window; 4] = [Window::Rectangular, Window::Hann, Window::Hamming, Window::Blackman];
const LEN: usize = 4096;
/// Sample rate that puts 1 kHz exactly on bin 82 of a 4096 record (about 50 kS/s).
const FS_BIN_CENTRED: f64 = LEN as f64 * 1000.0 / 82.0;

fn sine(cycles: f64, amplitude: f64) -> Vec<f64> {
    (0..LEN)
        .map(|n| amplitude * (TAU * cycles * n as f64 / LEN as f64).sin())
        .collect()
}

fn square(cycles: f64) -> Vec<f64> {
    (0..LEN)
        .map(|n| {
            if (cycles * n as f64 / LEN as f64).fract() < 0.5 {
                1.0
            } else {
                -1.0
            }
        })
        .collect()
}

fn db(level: f64, reference: f64) -> f64 {
    20.0 * (level / reference).log10()
}

#[test]
fn given_a_bin_centred_sine_when_every_window_runs_then_the_peak_reads_its_rms_within_half_a_percent() {
    for window in WINDOWS {
        let result = spectrum(&sine(82.0, 1.0), window, FS_BIN_CENTRED).unwrap();
        let peak = peak_bin(&result, window).unwrap();
        assert_eq!(peak, 82, "{window:?}");
        assert!((result.rms[peak] / (1.0 / SQRT_2) - 1.0).abs() < 0.005, "{window:?}");
        assert!((peak as f64 * result.bin_hz - 1000.0).abs() < 1e-6, "{window:?}");
    }
}

#[test]
fn given_a_constant_input_when_transformed_then_only_the_dc_bin_is_non_zero_and_equals_it() {
    for window in WINDOWS {
        let result = spectrum(&vec![0.75; 1024], window, 1000.0).unwrap();
        assert!((result.rms[0] - 0.75).abs() < 1e-9, "{window:?}");
        let lobe = window.last_skipped_bin();
        assert!(result.rms[lobe + 1..].iter().all(|&level| level < 1e-9), "{window:?}");
    }
}

#[test]
fn given_a_negative_constant_when_transformed_then_the_dc_bin_is_its_magnitude() {
    let result = spectrum(&[-0.5; 64], Window::Hann, 1000.0).unwrap();
    assert!((result.rms[0] - 0.5).abs() < 1e-9);
}

#[test]
fn given_a_nyquist_alternation_when_transformed_then_the_last_bin_reads_its_rms() {
    let signal: Vec<f64> = (0..64).map(|n| if n % 2 == 0 { 1.0 } else { -1.0 }).collect();
    let result = spectrum(&signal, Window::Rectangular, 1000.0).unwrap();
    assert_eq!(result.rms.len(), 33);
    assert!((result.rms[32] - 1.0).abs() < 1e-9);
    assert!(result.rms[..32].iter().all(|&level| level < 1e-9));
}

#[test]
fn given_3000_samples_when_transformed_then_it_is_not_a_power_of_two() {
    assert_eq!(
        spectrum(&vec![0.0; 3000], Window::Hann, 1.0),
        Err(FftError::NotPowerOfTwo { len: 3000 })
    );
}

#[test]
fn given_fewer_than_eight_samples_when_transformed_then_it_is_too_short() {
    assert_eq!(
        spectrum(&[0.0; 4], Window::Hann, 1.0),
        Err(FftError::TooShort { len: 4 })
    );
    assert_eq!(spectrum(&[], Window::Hann, 1.0), Err(FftError::TooShort { len: 0 }));
}

#[test]
fn given_an_irregular_record_when_transformed_then_every_bin_matches_a_direct_dft() {
    let signal: Vec<f64> = (0..16).map(|n| ((n * 7 + 3) % 11) as f64 - 5.0).collect();
    let result = spectrum(&signal, Window::Rectangular, 16.0).unwrap();
    for (k, &level) in result.rms.iter().enumerate().take(8).skip(1) {
        let (mut re, mut im) = (0.0, 0.0);
        for (n, v) in signal.iter().enumerate() {
            let angle = TAU * (k * n) as f64 / 16.0;
            re += v * angle.cos();
            im -= v * angle.sin();
        }
        let expected = re.hypot(im) / 16.0 * SQRT_2;
        assert!((level - expected).abs() < 1e-12, "bin {k}");
    }
}

#[test]
fn given_a_1_khz_square_when_transformed_then_odd_harmonics_follow_one_over_n_and_even_ones_are_low() {
    let result = spectrum(&square(82.0), Window::Hann, FS_BIN_CENTRED).unwrap();
    let fundamental = result.rms[82];
    let near = |bin: usize| result.rms[bin - 1..=bin + 1].iter().copied().fold(0.0, f64::max);
    for (harmonic, bin) in [(3.0, 246), (5.0, 410), (7.0, 574)] {
        assert!(
            (near(bin) / fundamental * harmonic - 1.0).abs() < 0.1,
            "harmonic {harmonic}"
        );
    }
    for bin in [164, 328, 492] {
        assert!(db(near(bin), fundamental) < -40.0, "even bin {bin}");
    }
}

#[test]
fn given_hann_and_a_dominant_dc_when_the_peak_is_taken_then_bins_zero_to_two_are_ignored() {
    let spectrum = Spectrum {
        bin_hz: 1.0,
        rms: vec![9.0, 8.0, 7.0, 0.5, 0.9, 0.1],
    };
    assert_eq!(peak_bin(&spectrum, Window::Hann), Some(4));
}

#[test]
fn given_each_window_when_the_peak_is_taken_then_the_lobe_skip_is_one_two_two_three_bins() {
    let spectrum = Spectrum {
        bin_hz: 1.0,
        rms: vec![5.0, 4.0, 3.0, 2.0, 1.0, 0.0],
    };
    let peaks: Vec<_> = WINDOWS.iter().map(|&window| peak_bin(&spectrum, window)).collect();
    assert_eq!(peaks, [Some(2), Some(3), Some(3), Some(4)]);
}

#[test]
fn given_no_bins_past_the_lobe_when_the_peak_is_taken_then_there_is_none() {
    let spectrum = Spectrum {
        bin_hz: 1.0,
        rms: vec![1.0, 1.0, 1.0],
    };
    assert_eq!(peak_bin(&spectrum, Window::Hann), None);
}

#[test]
fn given_random_records_when_rectangular_transformed_then_parseval_holds_within_one_percent() {
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
    let mut next = || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (state >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
    };
    for len in [8, 64, 512, 4096] {
        let signal: Vec<f64> = (0..len).map(|_| next()).collect();
        let mean_square = signal.iter().map(|v| v * v).sum::<f64>() / len as f64;
        let result = spectrum(&signal, Window::Rectangular, 1000.0).unwrap();
        let spectral: f64 = result.rms.iter().map(|v| v * v).sum();
        assert!((spectral / mean_square - 1.0).abs() < 0.01, "len {len}");
    }
}

#[test]
fn given_a_sample_rate_when_transformed_then_bin_spacing_is_rate_over_length() {
    let result = spectrum(&vec![0.0; LEN], Window::Hann, 50_000.0).unwrap();
    assert!((result.bin_hz - 50_000.0 / LEN as f64).abs() < 1e-12);
}
