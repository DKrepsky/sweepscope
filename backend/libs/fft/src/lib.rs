//! Single-sided amplitude spectrum with an own radix-2 transform (ADR 0018).
#![forbid(unsafe_code)]

mod transform;
mod window;

use std::fmt;

pub use window::Window;

const MIN_LEN: usize = 8;

#[derive(Debug, Clone, PartialEq)]
pub struct Spectrum {
    pub bin_hz: f64,
    /// `N / 2 + 1` bins, DC to Nyquist, rms volts, window gain corrected.
    pub rms: Vec<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FftError {
    NotPowerOfTwo { len: usize },
    TooShort { len: usize },
}

impl fmt::Display for FftError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotPowerOfTwo { len } => write!(f, "length {len} is not a power of two"),
            Self::TooShort { len } => write!(f, "length {len} is shorter than {MIN_LEN}"),
        }
    }
}

impl std::error::Error for FftError {}

pub fn spectrum(volts: &[f64], window: Window, sample_rate_hz: f64) -> Result<Spectrum, FftError> {
    let len = volts.len();
    if len < MIN_LEN {
        return Err(FftError::TooShort { len });
    }
    if !len.is_power_of_two() {
        return Err(FftError::NotPowerOfTwo { len });
    }
    let weights = window.weights(len);
    let gain: f64 = weights.iter().sum();
    let mut re: Vec<f64> = volts.iter().zip(&weights).map(|(v, w)| v * w).collect();
    let mut im = vec![0.0; len];
    transform::forward(&mut re, &mut im);
    let half = len / 2;
    let rms = (0..=half)
        .map(|k| {
            let magnitude = re[k].hypot(im[k]) / gain;
            let is_single_bin = k == 0 || k == half;
            if is_single_bin {
                magnitude
            } else {
                magnitude * std::f64::consts::SQRT_2
            }
        })
        .collect();
    Ok(Spectrum {
        bin_hz: sample_rate_hz / len as f64,
        rms,
    })
}

/// Highest bin after skipping DC and the main lobe of the window; the first one wins a tie.
pub fn peak_bin(spectrum: &Spectrum, window: Window) -> Option<usize> {
    let first = window.last_skipped_bin() + 1;
    spectrum
        .rms
        .iter()
        .enumerate()
        .skip(first)
        .filter(|(_, level)| level.is_finite())
        .fold(None, |best: Option<(usize, f64)>, (bin, &level)| match best {
            Some((_, top)) if top >= level => best,
            _ => Some((bin, level)),
        })
        .map(|(bin, _)| bin)
}

#[cfg(test)]
mod tests;
