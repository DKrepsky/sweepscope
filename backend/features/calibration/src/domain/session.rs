//! The dialog's state machine. It holds the results of finished channels until the fourth is accepted (ADR 0021 item 5).
use engineering_format::{format, Quantity};
use sweepscope_kernel::contracts::FormattedValue;

use crate::{OffsetRow, ZeroSession, ZeroStep};

const CHANNELS: usize = 4;

/// The command does not fit the step the session is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WrongStep(pub ZeroStep);

#[derive(Debug, Clone, PartialEq)]
pub enum Accepted {
    NextChannel,
    /// All four channels are accepted: write them in one go, then call `committed` or `fail`.
    Complete([Vec<f64>; CHANNELS]),
}

pub struct SessionMachine {
    ranges: Vec<f64>,
    step: ZeroStep,
    channel: usize,
    results: [Option<Vec<f64>>; CHANNELS],
    progress: Option<f64>,
    not_shorted: Option<Vec<String>>,
    failure: Option<String>,
}

impl SessionMachine {
    pub fn new(range_volts_per_div: Vec<f64>) -> Self {
        Self {
            ranges: range_volts_per_div,
            step: ZeroStep::Intro,
            channel: 0,
            results: Default::default(),
            progress: None,
            not_shorted: None,
            failure: None,
        }
    }

    pub fn step(&self) -> ZeroStep {
        self.step
    }

    pub fn start(&mut self) -> Result<(), WrongStep> {
        self.require(&[ZeroStep::Intro])?;
        self.step = ZeroStep::Channel;
        Ok(())
    }

    /// Returns the zero-based channel to measure.
    pub fn measure(&mut self) -> Result<usize, WrongStep> {
        self.require(&[ZeroStep::Channel, ZeroStep::NotShorted])?;
        self.step = ZeroStep::Measuring;
        self.progress = Some(0.0);
        self.not_shorted = None;
        Ok(self.channel)
    }

    pub fn set_progress(&mut self, progress: f64) {
        if self.step == ZeroStep::Measuring {
            self.progress = Some(progress.clamp(0.0, 1.0));
        }
    }

    pub fn accept(&mut self, offsets: Vec<f64>) -> Result<Accepted, WrongStep> {
        self.require(&[ZeroStep::Measuring])?;
        self.results[self.channel] = Some(offsets);
        if self.channel + 1 < CHANNELS {
            self.channel += 1;
            self.step = ZeroStep::Channel;
            self.progress = None;
            return Ok(Accepted::NextChannel);
        }
        Ok(Accepted::Complete(self.results.clone().map(Option::unwrap_or_default)))
    }

    /// Keeps the channel and the results of earlier ones; the user can retry.
    pub fn reject(&mut self, lines: Vec<String>) -> Result<(), WrongStep> {
        self.require(&[ZeroStep::Measuring])?;
        self.step = ZeroStep::NotShorted;
        self.progress = None;
        self.not_shorted = Some(lines);
        Ok(())
    }

    pub fn committed(&mut self) -> Result<(), WrongStep> {
        self.require(&[ZeroStep::Measuring])?;
        self.step = ZeroStep::Done;
        self.progress = None;
        Ok(())
    }

    pub fn fail(&mut self, text: impl Into<String>) {
        self.step = ZeroStep::Failed;
        self.progress = None;
        self.failure = Some(text.into());
    }

    /// Forgets everything measured so far; nothing was written before the commit.
    pub fn cancel(&mut self) {
        self.results = Default::default();
    }

    pub fn view(&self) -> ZeroSession {
        let in_turn = !matches!(self.step, ZeroStep::Intro | ZeroStep::Done);
        ZeroSession {
            step: self.step,
            channel: in_turn.then_some(self.channel as u8 + 1),
            done_channels: self.results.each_ref().map(Option::is_some),
            progress: self.progress,
            not_shorted: self.not_shorted.clone(),
            failure: self.failure.clone(),
            table: (self.step == ZeroStep::Done).then(|| self.table()),
        }
    }

    fn require(&self, allowed: &[ZeroStep]) -> Result<(), WrongStep> {
        if allowed.contains(&self.step) {
            Ok(())
        } else {
            Err(WrongStep(self.step))
        }
    }

    fn table(&self) -> Vec<OffsetRow> {
        self.ranges
            .iter()
            .enumerate()
            .map(|(index, &range)| OffsetRow {
                range: formatted(range, Quantity::VoltsPerDiv),
                offsets: self.results.each_ref().map(|result| {
                    let volts = result.as_ref()?.get(index)?;
                    Some(offset_cell(*volts))
                }),
            })
            .collect()
    }
}

/// A stored zero reads `0 V` (design), not the readout's `0.00 V`.
fn offset_cell(volts: f64) -> FormattedValue {
    if volts == 0.0 {
        return FormattedValue {
            value: 0.0,
            text: "0".to_owned(),
            unit: "V".to_owned(),
        };
    }
    formatted(volts, Quantity::Volts)
}

fn formatted(value: f64, quantity: Quantity) -> FormattedValue {
    let shown = format(value, quantity);
    FormattedValue {
        value: shown.value,
        text: shown.text,
        unit: shown.unit,
    }
}
