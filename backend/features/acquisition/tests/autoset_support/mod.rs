#![allow(dead_code)]
//! A synthetic scope input: turns an applied setup into the frame the device would capture of a known signal.
use std::f64::consts::TAU;
use std::sync::{Arc, Mutex};

use sweepscope_kernel::contracts::{AcquisitionSetup, ChannelData, Frame, TriggerOutcome};

use crate::domain_support::capabilities;

const COUNTS_PER_DIV: f64 = 24.15;
const CENTRE_CODE: f64 = 128.0;
const CAPTURE_PHASE: f64 = 0.37;

#[derive(Debug, Clone, Copy)]
pub struct Signal {
    pub frequency_hz: f64,
    pub peak_to_peak_volts: f64,
}

impl Signal {
    pub fn sine(frequency_hz: f64, peak_to_peak_volts: f64) -> Self {
        Self {
            frequency_hz,
            peak_to_peak_volts,
        }
    }

    pub fn flat() -> Self {
        Self::sine(1e3, 0.0)
    }

    fn volts_at(&self, seconds: f64) -> f64 {
        0.5 * self.peak_to_peak_volts * (TAU * seconds * self.frequency_hz).sin()
    }
}

/// The signal every enabled channel reads; it can change while a test runs.
#[derive(Clone)]
pub struct Input(Arc<Mutex<Signal>>);

impl Input {
    pub fn new(signal: Signal) -> Self {
        Self(Arc::new(Mutex::new(signal)))
    }

    pub fn set(&self, signal: Signal) {
        *self.0.lock().unwrap() = signal;
    }

    pub fn frame(&self, setup: &AcquisitionSetup) -> Frame {
        let signal = *self.0.lock().unwrap();
        capture(signal, setup)
    }
}

pub fn capture(signal: Signal, setup: &AcquisitionSetup) -> Frame {
    let caps = capabilities();
    let enabled = setup.channels.iter().filter(|c| c.enabled).count() as u8;
    let interval = caps.sample_interval_s(enabled, setup.time_base_index);
    let start = CAPTURE_PHASE / signal.frequency_hz;
    let channel = |index: usize| {
        let channel = &setup.channels[index];
        channel.enabled.then(|| {
            let volts_per_div = caps.vertical_ranges[usize::from(channel.range_index)].volts_per_div;
            let zero_code = CENTRE_CODE + channel.position_divs * COUNTS_PER_DIV;
            let codes = (0..caps.record_len as usize)
                .map(|n| {
                    let volts = signal.volts_at(start + interval * n as f64);
                    (zero_code + volts / volts_per_div * COUNTS_PER_DIV)
                        .round()
                        .clamp(0.0, 255.0) as u8
                })
                .collect();
            ChannelData {
                codes,
                volts_per_div,
                zero_code,
            }
        })
    };
    Frame {
        channels: std::array::from_fn(channel),
        counts_per_div: COUNTS_PER_DIV,
        sample_interval_s: interval,
        trigger: TriggerOutcome::Auto,
    }
}
