//! The sweep of one channel over every range: gate range first, then ascending (ADR 0021, work plan 1.3).
use engineering_format::{format, Quantity};

use super::rules::{
    GATE_MAX_LEVEL_CODES, GATE_MAX_PEAK_TO_PEAK_CODES, GATE_MAX_STD_DEV_CODES, GATE_VOLTS_PER_DIV, NOISE_FACTOR,
    NOISE_FLOOR_CODES, OTHER_MAX_LEVEL_CODES, OTHER_MIN_LEVEL_VOLTS, REFERENCE_STD_DEV_CODES,
};
use super::statistics::Reading;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeasurePlan {
    pub range_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectionKind {
    SignalPresent,
    LevelOffZero,
    TooNoisy,
}

/// Why the input is not a short: the kind for logic, the line the dialog shows, with measured numbers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rejection {
    pub kind: RejectionKind,
    pub range_index: usize,
    pub line: String,
    /// False when the reading clipped: its offset would be wrong by an unknown amount.
    pub continuable: bool,
}

/// `accept_rejected` was called with no rejected reading to accept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NothingPending;

#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    Measure(MeasurePlan),
    /// Offset in volts per range index.
    Done(Vec<f64>),
    NotShorted(Rejection),
}

pub struct ZeroSweep {
    ranges: Vec<f64>,
    gate: Option<usize>,
    order: Vec<usize>,
    cursor: usize,
    offsets: Vec<f64>,
    /// The rejected reading of the range at the cursor, kept for `accept_rejected`.
    pending: Option<Reading>,
    /// Set once a rejection was accepted: only clipped readings are refused from then on.
    forced: bool,
}

impl ZeroSweep {
    pub fn new(volts_per_div: Vec<f64>) -> Self {
        let gate = gate_index(&volts_per_div);
        let order = gate
            .into_iter()
            .chain((0..volts_per_div.len()).filter(|&i| Some(i) != gate))
            .collect();
        Self {
            offsets: vec![0.0; volts_per_div.len()],
            ranges: volts_per_div,
            gate,
            order,
            cursor: 0,
            pending: None,
            forced: false,
        }
    }

    /// Share of the ranges accepted so far, 0 to 1.
    pub fn progress(&self) -> f64 {
        if self.order.is_empty() {
            return 1.0;
        }
        self.cursor as f64 / self.order.len() as f64
    }

    /// Feed `None` to get the plan to measure now, then the reading of that plan to get the next step.
    pub fn next(&mut self, reading: Option<&Reading>) -> Step {
        if let (Some(reading), Some(&index)) = (reading, self.order.get(self.cursor)) {
            if let Some(rejection) = self.judge(index, reading) {
                self.pending = Some(*reading);
                return Step::NotShorted(rejection);
            }
            self.store(index, reading);
        }
        self.advance()
    }

    /// Stores the offset of the rejected reading, stops checking, and goes on with the next range.
    pub fn accept_rejected(&mut self) -> Result<Step, NothingPending> {
        let reading = self.pending.take().ok_or(NothingPending)?;
        let &index = self.order.get(self.cursor).ok_or(NothingPending)?;
        self.store(index, &reading);
        self.forced = true;
        Ok(self.advance())
    }

    fn store(&mut self, index: usize, reading: &Reading) {
        self.offsets[index] = reading.codes_to_volts(reading.offset_codes(), self.ranges[index]);
        self.cursor += 1;
        self.pending = None;
    }

    fn advance(&self) -> Step {
        match self.order.get(self.cursor) {
            Some(&range_index) => Step::Measure(MeasurePlan { range_index }),
            None => Step::Done(self.offsets.clone()),
        }
    }

    fn judge(&self, index: usize, reading: &Reading) -> Option<Rejection> {
        let range = self.ranges[index];
        let found = if self.forced {
            judge_clipped(reading, range)
        } else if self.gate == Some(index) {
            judge_gate(reading, range)
        } else {
            judge_other(reading, range, index)
        };
        found.map(|(kind, line)| Rejection {
            kind,
            range_index: index,
            line,
            continuable: !reading.stats.clipped,
        })
    }
}

fn gate_index(ranges: &[f64]) -> Option<usize> {
    ranges
        .iter()
        .enumerate()
        .min_by(|a, b| distance_to_gate(*a.1).total_cmp(&distance_to_gate(*b.1)))
        .map(|(index, _)| index)
}

fn distance_to_gate(range: f64) -> f64 {
    (range / GATE_VOLTS_PER_DIV).ln().abs()
}

type Found = Option<(RejectionKind, String)>;

fn judge_clipped(reading: &Reading, range: f64) -> Found {
    reading.stats.clipped.then(|| {
        let line = format!("Input clipped at {}.", range_text(range));
        (RejectionKind::SignalPresent, line)
    })
}

fn judge_gate(reading: &Reading, range: f64) -> Found {
    let stats = &reading.stats;
    if let Some(clipped) = judge_clipped(reading, range) {
        return Some(clipped);
    }
    if stats.peak_to_peak > GATE_MAX_PEAK_TO_PEAK_CODES {
        let line = measured_line(
            "Peak to peak",
            stats.peak_to_peak,
            GATE_MAX_PEAK_TO_PEAK_CODES,
            reading,
            range,
        );
        return Some((RejectionKind::SignalPresent, line));
    }
    if stats.std_dev > GATE_MAX_STD_DEV_CODES {
        let line = noise_line(stats.std_dev, GATE_MAX_STD_DEV_CODES, reading, range);
        return Some((RejectionKind::SignalPresent, line));
    }
    level_check(reading, range, GATE_MAX_LEVEL_CODES)
}

fn judge_other(reading: &Reading, range: f64, index: usize) -> Found {
    let reference = REFERENCE_STD_DEV_CODES.get(index).copied().unwrap_or(0.0);
    let limit = NOISE_FLOOR_CODES.max(NOISE_FACTOR * reference);
    if reading.stats.std_dev > limit {
        let line = noise_line(reading.stats.std_dev, limit, reading, range);
        return Some((RejectionKind::TooNoisy, line));
    }
    let limit_codes = OTHER_MAX_LEVEL_CODES.max(OTHER_MIN_LEVEL_VOLTS / range * reading.counts_per_div);
    level_check(reading, range, limit_codes)
}

fn level_check(reading: &Reading, range: f64, limit_codes: f64) -> Found {
    let level = reading.offset_codes().abs();
    (level > limit_codes).then(|| {
        let line = format!(
            "Level {} from zero at {}, limit {}.",
            volts_text(reading.codes_to_volts(level, range)),
            range_text(range),
            volts_text(reading.codes_to_volts(limit_codes, range)),
        );
        (RejectionKind::LevelOffZero, line)
    })
}

fn measured_line(name: &str, codes: f64, limit_codes: f64, reading: &Reading, range: f64) -> String {
    format!(
        "{name} {} at {}, limit {}.",
        volts_text(reading.codes_to_volts(codes, range)),
        range_text(range),
        volts_text(reading.codes_to_volts(limit_codes, range)),
    )
}

fn noise_line(std_dev: f64, limit: f64, reading: &Reading, range: f64) -> String {
    format!(
        "Noise {} rms at {}, limit {}.",
        volts_text(reading.codes_to_volts(std_dev, range)),
        range_text(range),
        volts_text(reading.codes_to_volts(limit, range)),
    )
}

fn volts_text(volts: f64) -> String {
    let shown = format(volts, Quantity::Volts);
    format!("{} {}", shown.text, shown.unit)
}

fn range_text(range: f64) -> String {
    let shown = format(range, Quantity::VoltsPerDiv);
    format!("{} {}", shown.text, shown.unit)
}
