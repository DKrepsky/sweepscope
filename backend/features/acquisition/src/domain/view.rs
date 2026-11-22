//! Builds the published slices (texts, availability) from the model; no state of its own.
use std::time::Instant;

use engineering_format::{format, Quantity};
use sweepscope_kernel::contracts::{Capabilities, Slope};

use super::meter::CaptureMeter;
use super::model::{Model, CHANNELS};
use crate::{
    Availability, ChannelState, ControlsAvailability, FormattedValue, HorizontalState, InstrumentFacts,
    InstrumentState, ScaleChoice, TriggerState,
};

pub const NO_SCOPE: &str = "No scope connected";
pub const ROLL_BLOCKED: &str = "Roll mode arrives in a later release";
pub const LAST_CHANNEL: &str = "At least one channel must stay on";
const NOT_RUNNING: &str = "Start acquisition first";
const RANGE_LOW_LIMIT_V: f64 = 0.05;
const RANGE_HIGH_LIMIT_V: f64 = 1.0;

pub fn formatted(value: f64, quantity: Quantity) -> FormattedValue {
    let f = format(value, quantity);
    FormattedValue {
        value: f.value,
        text: f.text,
        unit: f.unit,
    }
}

fn available() -> Availability {
    Availability {
        enabled: true,
        reason: None,
    }
}

fn unavailable(reason: &str) -> Availability {
    Availability {
        enabled: false,
        reason: Some(reason.to_owned()),
    }
}

fn availability(enabled: bool, reason: &str) -> Availability {
    if enabled {
        available()
    } else {
        unavailable(reason)
    }
}

/// `reason` is the connection text while no device is ready; it overrides every availability.
pub fn instrument_state(caps: &Capabilities, model: &Model, revision: u64, reason: Option<&str>) -> InstrumentState {
    let gate = |own: Availability| reason.map_or(own, unavailable);
    let controls = ControlsAvailability {
        run: gate(available()),
        force: gate(availability(model.running, NOT_RUNNING)),
        vertical: gate(available()),
        horizontal: gate(available()),
        trigger: gate(available()),
    };
    InstrumentState {
        revision,
        running: model.running,
        channels: (0..CHANNELS).map(|n| channel_state(caps, model, n, &gate)).collect(),
        horizontal: horizontal_state(caps, model, &gate),
        trigger: trigger_state(caps, model),
        controls,
    }
}

fn unverified_text(volts_per_div: f64) -> Option<String> {
    let side = if volts_per_div < RANGE_LOW_LIMIT_V {
        "below 50 mV"
    } else if volts_per_div > RANGE_HIGH_LIMIT_V {
        "above 1 V"
    } else {
        return None;
    };
    Some(format!(
        "Unverified range: {side} per div. Accuracy up to about 5 % of a division."
    ))
}

fn channel_state(
    caps: &Capabilities,
    model: &Model,
    number: usize,
    gate: &impl Fn(Availability) -> Availability,
) -> ChannelState {
    let setup = &model.setup.channels[number];
    let probe = model.probes[number];
    let index = usize::from(setup.range_index);
    let range = caps.vertical_ranges.get(index);
    let volts_per_div = range.map_or(0.0, |r| r.volts_per_div);
    let only_enabled = setup.enabled && model.enabled_count() == 1;
    ChannelState {
        number: number as u8 + 1,
        enabled: setup.enabled,
        range: ScaleChoice {
            index: setup.range_index,
            label: formatted(volts_per_div * probe, Quantity::VoltsPerDiv),
            verified: range.is_none_or(|r| r.verified),
            faster: gate(availability(index > 0, "Smallest range")),
            slower: gate(availability(index + 1 < caps.vertical_ranges.len(), "Largest range")),
        },
        position: formatted(setup.position_divs, Quantity::Divisions),
        position_divs: setup.position_divs,
        max_position_divs: range.map_or(0.0, |r| r.max_position_divs),
        coupling: setup.coupling,
        bandwidth_limit: setup.bandwidth_limit,
        probe,
        probe_text: format!("{probe}x"),
        invert: model.invert[number],
        unverified: unverified_text(volts_per_div),
        enable: gate(availability(!only_enabled, LAST_CHANNEL)),
    }
}

fn horizontal_state(
    caps: &Capabilities,
    model: &Model,
    gate: &impl Fn(Availability) -> Availability,
) -> HorizontalState {
    let index = usize::from(model.setup.time_base_index);
    let step = caps.time_bases.get(index);
    let slower_is_roll = caps.time_bases.get(index + 1).is_some_and(|t| t.streaming);
    let slower = if slower_is_roll {
        unavailable(ROLL_BLOCKED)
    } else {
        availability(index + 1 < caps.time_bases.len(), "Slowest time base")
    };
    let g = model.geometry(caps);
    let record_len = f64::from(caps.record_len);
    let window_divs = (g.window_len - 1) as f64 / g.samples_per_div;
    HorizontalState {
        time_base: ScaleChoice {
            index: model.setup.time_base_index,
            label: formatted(step.map_or(0.0, |t| t.seconds_per_div), Quantity::SecondsPerDiv),
            verified: step.is_none_or(|t| t.verified),
            faster: gate(availability(index > 0, "Fastest time base")),
            slower: gate(slower),
        },
        trigger_position: formatted(model.trigger_fraction * 100.0, Quantity::Percent),
        trigger_position_fraction: model.trigger_fraction,
        window_text: format!("{window_divs:.1} of {:.1} div", g.record_divs),
        window_start_fraction: g.window_start as f64 / record_len,
        window_size_fraction: g.window_len as f64 / record_len,
        trigger_tick_fraction: model.setup.trigger.horizontal_position_percent / 100.0,
    }
}

fn trigger_state(caps: &Capabilities, model: &Model) -> TriggerState {
    let trigger = &model.setup.trigger;
    let source = usize::from(trigger.source);
    let level = formatted(trigger.level_volts * model.probes[source], Quantity::Volts);
    let arrow = match trigger.slope {
        Slope::Rising => "\u{2197}",
        Slope::Falling => "\u{2198}",
    };
    let position = formatted(model.trigger_fraction * 100.0, Quantity::Percent);
    TriggerState {
        source: trigger.source + 1,
        slope: trigger.slope,
        sweep: trigger.sweep,
        level_tag: format!("T{} {arrow} {} {}", trigger.source + 1, level.text, level.unit),
        level,
        level_divisions: model.level_divisions(caps),
        position_tag: format!("{} {}", position.text, position.unit),
    }
}

pub fn instrument_facts(
    caps: &Capabilities,
    model: &Model,
    meter: &CaptureMeter,
    now: Instant,
    revision: u64,
) -> InstrumentFacts {
    let interval = caps.sample_interval_s(model.enabled_count(), model.setup.time_base_index);
    let record_duration = f64::from(caps.record_len) * interval;
    InstrumentFacts {
        revision,
        sample_rate: Some(formatted(1.0 / interval, Quantity::SamplesPerSecond)),
        sample_interval: Some(formatted(interval, Quantity::Seconds)),
        record_points: Some(formatted(f64::from(caps.record_len), Quantity::Points)),
        capture_rate: meter.rate(now).map(|rate| formatted(rate, Quantity::Hertz)),
        dead_time: meter
            .dead_time_percent(now, record_duration)
            .map(|percent| formatted(percent, Quantity::Percent)),
    }
}
