//! Synthetic device: turns a plan into what the scope would have captured of a known signal.

use autoset::{Autoset, ChannelObservation, Observation, Plan, Step, Tables};

pub const COUNTS_PER_DIV: f64 = 25.0;
pub const RECORD_LEN: usize = 4096;
const SAMPLES_PER_DIV: f64 = 250.0;
const MIN_SWING_COUNTS: f64 = 10.0;
const PERIOD_NEEDS_PERIODS: f64 = 1.5;
const CAPTURE_PHASE: f64 = 0.37;

#[derive(Clone, Copy)]
pub enum Shape {
    Square,
    Sine,
}

#[derive(Clone, Copy)]
pub struct Signal {
    pub shape: Shape,
    pub frequency_hz: f64,
    pub peak_to_peak_volts: f64,
    pub offset_volts: f64,
}

impl Signal {
    pub fn square(frequency_hz: f64, peak_to_peak_volts: f64) -> Self {
        Self {
            shape: Shape::Square,
            frequency_hz,
            peak_to_peak_volts,
            offset_volts: 0.0,
        }
    }

    pub fn sine(frequency_hz: f64, peak_to_peak_volts: f64) -> Self {
        Self {
            shape: Shape::Sine,
            frequency_hz,
            peak_to_peak_volts,
            offset_volts: 0.0,
        }
    }

    fn volts_at(&self, seconds: f64) -> f64 {
        let cycle = (seconds * self.frequency_hz).fract();
        let unit = match self.shape {
            Shape::Square => {
                if cycle < 0.5 {
                    0.5
                } else {
                    -0.5
                }
            }
            Shape::Sine => 0.5 * (std::f64::consts::TAU * cycle).sin(),
        };
        self.offset_volts + unit * self.peak_to_peak_volts
    }
}

/// Tables of the 6254BC (device profile): 2 mV to 10 V, 2 ns to 1000 s, streaming from 500 ms.
pub fn tables() -> Tables {
    let mut volts = vec![0.002, 0.005];
    for decade in [0.01, 0.1, 1.0] {
        volts.extend([1.0, 2.0, 5.0].map(|mantissa| mantissa * decade));
    }
    volts.push(10.0);
    let mut seconds = vec![2e-9, 5e-9];
    for exponent in -8..=2 {
        seconds.extend([1.0, 2.0, 5.0].map(|mantissa| mantissa * 10f64.powi(exponent)));
    }
    seconds.push(1000.0);
    Tables {
        range_verified: (0..volts.len()).map(|index| (4..=8).contains(&index)).collect(),
        max_position_divs: volts.iter().map(|_| 4.0).collect(),
        range_volts_per_div: volts,
        time_base_s_per_div: seconds,
        roll_from_index: 25,
        counts_per_div: COUNTS_PER_DIV,
        record_len: RECORD_LEN,
        samples_per_div: SAMPLES_PER_DIV,
    }
}

pub fn start_plan(tables: &Tables) -> Plan {
    let one_volt = tables.range_volts_per_div.iter().position(|v| *v == 1.0).unwrap();
    let one_ms = tables
        .time_base_s_per_div
        .iter()
        .position(|s| (*s - 1e-3).abs() < 1e-12)
        .unwrap();
    Plan {
        range_index: [one_volt as u8; 4],
        position_divs: [0.0; 4],
        time_base_index: one_ms as u8,
        trigger_source: 0,
        trigger_level_volts: 0.0,
    }
}

pub struct Run {
    pub step: Step,
    pub probes: usize,
    pub tables: Tables,
}

impl Run {
    pub fn plan(&self) -> &Plan {
        match &self.step {
            Step::Done(plan) | Step::NoSignal(plan) | Step::Probe(plan) => plan,
            Step::Failed(_) => panic!("run failed: {:?}", self.step),
        }
    }

    pub fn screen_periods(&self, signal: &Signal) -> f64 {
        let seconds = self.tables.time_base_s_per_div[self.plan().time_base_index as usize];
        10.0 * seconds * signal.frequency_hz
    }

    pub fn screen_divs(&self, signal: &Signal) -> f64 {
        let range = self.plan().range_index[0] as usize;
        signal.peak_to_peak_volts / self.tables.range_volts_per_div[range]
    }
}

pub fn run_on_channel_one(signal: Signal) -> Run {
    run_with(signal, [true, false, false, false])
}

pub fn run_with(signal: Signal, enabled: [bool; 4]) -> Run {
    drive(
        enabled,
        |_, plan, tables| observe(&signal, plan, tables),
        |autoset| autoset,
    )
}

pub fn drive(
    enabled: [bool; 4],
    mut device: impl FnMut(usize, &Plan, &Tables) -> Observation,
    configure: impl FnOnce(Autoset) -> Autoset,
) -> Run {
    let tables = tables();
    let start = start_plan(&tables);
    let mut autoset = configure(Autoset::new(tables.clone(), enabled, start));
    let mut step = autoset.first();
    let mut probes = 1;
    while let Step::Probe(plan) = &step {
        assert!(probes <= 14, "more than 14 probes");
        let observation = device(probes, plan, &tables);
        step = autoset.next(&observation);
        if matches!(step, Step::Probe(_)) {
            probes += 1;
        }
    }
    Run { step, probes, tables }
}

pub fn observe(signal: &Signal, plan: &Plan, tables: &Tables) -> Observation {
    let seconds_per_div = tables.time_base_s_per_div[plan.time_base_index as usize];
    let interval = seconds_per_div / SAMPLES_PER_DIV;
    let span = interval * RECORD_LEN as f64;
    let start = CAPTURE_PHASE / signal.frequency_hz;
    let range = plan.range_index[0] as usize;
    let volts_per_div = tables.range_volts_per_div[range];
    let codes: Vec<u8> = (0..RECORD_LEN)
        .map(|index| {
            let volts = signal.volts_at(start + interval * index as f64);
            let code = 128.0 + (volts / volts_per_div + plan.position_divs[0]) * COUNTS_PER_DIV;
            code.round().clamp(0.0, 255.0) as u8
        })
        .collect();
    let (min_code, max_code) = (*codes.iter().min().unwrap(), *codes.iter().max().unwrap());
    let swing = f64::from(max_code) - f64::from(min_code);
    let period = (swing >= MIN_SWING_COUNTS && span >= PERIOD_NEEDS_PERIODS / signal.frequency_hz)
        .then(|| 1.0 / signal.frequency_hz);
    let channel = ChannelObservation {
        min_code,
        max_code,
        clipped: min_code == 0 || max_code == 255,
        period_s: period,
    };
    Observation {
        channels: [Some(channel), None, None, None],
        sample_interval_s: interval,
    }
}
