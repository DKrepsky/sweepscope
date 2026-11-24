//! One auto-setup run (ADR 0020): the decision machine, its budget, the status it publishes and the plan-to-setup rules.
use std::time::{Duration, Instant};

use autoset::{Autoset, ChannelObservation, Observation, Step, Tables};
use engineering_format::Quantity;
use measurements::{is_clipped, measure};
use sweepscope_kernel::contracts::{AcquisitionSetup, Capabilities, Frame, Slope, Sweep};

use super::model::{Model, CHANNELS};
use super::view::formatted;
use crate::{AutosetOutcome, AutosetOutcomeKind, AutosetStatus};

pub use autoset::Plan;

/// The lib has no clock; the wall-time half of its budget is kept here (ADR 0020 item 2).
const BUDGET: Duration = Duration::from_secs(15);
/// A peak to peak under this many counts is a flat line (ADR 0020 item 4).
const SIGNAL_COUNTS: u8 = 3;
/// Capabilities carry no scale: the driver measured 21.1, 21.6 and 24.15 counts per division by ADC mode.
const NOMINAL_COUNTS_PER_DIV: f64 = 23.0;

pub const FAILED_TEXT: &str = "Auto-setup could not find a stable setting. Previous setup kept.";
pub const NO_SIGNAL_TEXT: &str = "Auto-setup: no signal on any enabled channel.";

/// What the machine needs to start.
pub struct Start {
    machine: Autoset,
    enabled: [bool; CHANNELS],
}

pub enum Next {
    Probe(Plan),
    Finish(Finish),
}

pub enum Finish {
    Done(Plan),
    NoSignal(Plan),
    Failed,
}

struct Active {
    machine: Autoset,
    enabled: [bool; CHANNELS],
    started: Instant,
    was_running: bool,
    waiting: Option<u64>,
}

#[derive(Default)]
pub struct AutosetRun {
    status: AutosetStatus,
    last_outcome_id: u64,
    active: Option<Active>,
}

impl AutosetRun {
    pub fn status(&self) -> AutosetStatus {
        self.status.clone()
    }

    pub fn active(&self) -> bool {
        self.active.is_some()
    }

    pub fn was_running(&self) -> bool {
        self.active.as_ref().is_some_and(|a| a.was_running)
    }

    pub fn waiting(&self) -> Option<u64> {
        self.active.as_ref().and_then(|a| a.waiting)
    }

    pub fn wait_for(&mut self, setup_revision: u64) {
        if let Some(active) = &mut self.active {
            active.waiting = Some(setup_revision);
        }
    }

    pub fn expired(&self, now: Instant) -> bool {
        self.active
            .as_ref()
            .is_some_and(|a| now.saturating_duration_since(a.started) >= BUDGET)
    }

    /// A new run clears the `No signal` tag; the previous outcome stays until this one has its own.
    pub fn begin(&mut self, start: Start, now: Instant, was_running: bool) -> Next {
        let Start { mut machine, enabled } = start;
        let first = machine.first();
        self.status.running = true;
        self.status.no_signal = false;
        self.active = Some(Active {
            machine,
            enabled,
            started: now,
            was_running,
            waiting: None,
        });
        next_of(first)
    }

    pub fn observe(&mut self, frame: &Frame) -> Option<Next> {
        let active = self.active.as_mut()?;
        let observation = observation(frame, &active.enabled);
        Some(next_of(active.machine.next(&observation)))
    }

    /// Ends the run and publishes its outcome once, under a new id.
    pub fn conclude(&mut self, kind: AutosetOutcomeKind, text: String) {
        self.active = None;
        self.last_outcome_id += 1;
        self.status.running = false;
        self.status.no_signal = kind == AutosetOutcomeKind::NoSignal;
        self.status.outcome = Some(AutosetOutcome {
            id: self.last_outcome_id,
            kind,
            text,
        });
    }

    /// The tag stays until an accepted frame shows a swing on an enabled channel.
    pub fn note_frame(&mut self, frame: &Frame) {
        if self.status.no_signal && has_signal(frame) {
            self.status.no_signal = false;
        }
    }
}

fn next_of(step: Step) -> Next {
    match step {
        Step::Probe(plan) => Next::Probe(plan),
        Step::Done(plan) => Next::Finish(Finish::Done(plan)),
        Step::NoSignal(plan) => Next::Finish(Finish::NoSignal(plan)),
        Step::Failed(_) => Next::Finish(Finish::Failed),
    }
}

fn swing(codes: &[u8]) -> u8 {
    let lowest = codes.iter().copied().min().unwrap_or(0);
    let highest = codes.iter().copied().max().unwrap_or(0);
    highest - lowest
}

fn has_signal(frame: &Frame) -> bool {
    frame
        .channels
        .iter()
        .flatten()
        .any(|data| swing(&data.codes) >= SIGNAL_COUNTS)
}

fn observation(frame: &Frame, enabled: &[bool; CHANNELS]) -> Observation {
    let channel = |index: usize| {
        let data = frame.channels[index].as_ref().filter(|_| enabled[index])?;
        let volts_per_count = data.volts_per_div / frame.counts_per_div;
        let volts: Vec<f64> = data
            .codes
            .iter()
            .map(|&code| (f64::from(code) - data.zero_code) * volts_per_count)
            .collect();
        let period = measure(&volts, frame.sample_interval_s, volts_per_count).period;
        Some(ChannelObservation {
            min_code: data.codes.iter().copied().min().unwrap_or(0),
            max_code: data.codes.iter().copied().max().unwrap_or(0),
            clipped: is_clipped(&data.codes),
            period_s: period.ok(),
        })
    };
    Observation {
        channels: std::array::from_fn(channel),
        sample_interval_s: frame.sample_interval_s,
    }
}

pub fn start(caps: &Capabilities, model: &Model) -> Start {
    let enabled = std::array::from_fn(|n| model.setup.channels[n].enabled);
    let current = Plan {
        range_index: std::array::from_fn(|n| model.setup.channels[n].range_index),
        position_divs: std::array::from_fn(|n| model.setup.channels[n].position_divs),
        time_base_index: model.setup.time_base_index,
        trigger_source: model.setup.trigger.source,
        trigger_level_volts: model.setup.trigger.level_volts,
    };
    Start {
        machine: Autoset::new(tables(caps), enabled, current),
        enabled,
    }
}

fn tables(caps: &Capabilities) -> Tables {
    let ranges = &caps.vertical_ranges;
    Tables {
        range_volts_per_div: ranges.iter().map(|r| r.volts_per_div).collect(),
        range_verified: ranges.iter().map(|r| r.verified).collect(),
        max_position_divs: ranges.iter().map(|r| r.max_position_divs).collect(),
        time_base_s_per_div: caps.time_bases.iter().map(|t| t.seconds_per_div).collect(),
        roll_from_index: caps
            .time_bases
            .iter()
            .position(|t| t.streaming)
            .unwrap_or(caps.time_bases.len()),
        counts_per_div: NOMINAL_COUNTS_PER_DIV,
        record_len: caps.record_len as usize,
        samples_per_div: caps.samples_per_div,
    }
}

/// Channels that are off keep what they had; the trigger becomes rising edge on Auto sweep as the plan decides.
pub fn apply_plan(caps: &Capabilities, model: &mut Model, plan: &Plan) {
    for number in 0..CHANNELS {
        if !model.setup.channels[number].enabled {
            continue;
        }
        model.setup.channels[number].range_index = plan.range_index[number];
        model.setup.channels[number].position_divs = plan.position_divs[number];
        model.clamp_position(caps, number);
    }
    model.setup.time_base_index = plan.time_base_index;
    model.setup.trigger.source = plan.trigger_source;
    model.setup.trigger.level_volts = plan.trigger_level_volts;
    model.setup.trigger.slope = Slope::Rising;
    model.setup.trigger.sweep = Sweep::Auto;
    model.set_pulse(false);
    model.settle(caps);
}

/// What the device runs for one probe: the plan on a copy, so the published setup stays untouched until the end.
pub fn probe_setup(caps: &Capabilities, model: &Model, plan: &Plan) -> AcquisitionSetup {
    let mut probe = model.clone();
    probe.single = false;
    apply_plan(caps, &mut probe, plan);
    probe.device_setup()
}

/// `CH1 500 mV/div, 200 µs/div`, from the setup the run ended with.
pub fn summary(caps: &Capabilities, model: &Model) -> String {
    let channels = (0..CHANNELS).filter(|n| model.setup.channels[*n].enabled).map(|n| {
        let volts = model.range(caps, n).map_or(0.0, |r| r.volts_per_div) * model.probes[n];
        let label = formatted(volts, Quantity::VoltsPerDiv);
        format!("CH{} {} {}", n + 1, label.text, label.unit)
    });
    let seconds = caps
        .time_bases
        .get(usize::from(model.setup.time_base_index))
        .map_or(0.0, |t| t.seconds_per_div);
    let time = formatted(seconds, Quantity::SecondsPerDiv);
    let parts: Vec<String> = channels.chain([format!("{} {}", time.text, time.unit)]).collect();
    format!("Auto-setup done. {}.", parts.join(", "))
}
