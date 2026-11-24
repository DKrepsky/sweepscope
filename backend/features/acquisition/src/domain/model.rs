//! The editable part of the instrument: what the user chose, validated against the capabilities.
use sweepscope_kernel::contracts::{
    AcquisitionSetup, Capabilities, ChannelSetup, Coupling, Slope, Sweep, TriggerKind, TriggerSetup, VerticalRange,
};

use super::defaults::Defaults;
use super::geometry::{geometry, record_percent, Geometry};

pub const CHANNELS: usize = 4;

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub setup: AcquisitionSetup,
    pub probes: [f64; CHANNELS],
    pub invert: [bool; CHANNELS],
    pub running: bool,
    /// Trigger position on the screen, 0 to 1; the hardware percent derives from it and the geometry.
    pub trigger_fraction: f64,
}

pub fn nearest_index<T>(items: &[T], wanted: f64, value_of: impl Fn(&T) -> f64) -> u8 {
    let distance = |item: &T| (value_of(item).ln() - wanted.ln()).abs();
    let best = items
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| distance(a).total_cmp(&distance(b)));
    best.map_or(0, |(index, _)| index as u8)
}

impl Model {
    pub fn initial(defaults: &Defaults, caps: &Capabilities) -> Self {
        let range_index = nearest_index(&caps.vertical_ranges, defaults.range_volts_per_div, |r| r.volts_per_div);
        let channel = |number: usize| ChannelSetup {
            enabled: number == 0,
            range_index,
            position_divs: defaults.position_divs,
            coupling: Coupling::Dc,
            bandwidth_limit: false,
        };
        let mut model = Self {
            setup: AcquisitionSetup {
                channels: std::array::from_fn(channel),
                time_base_index: nearest_index(&caps.time_bases, defaults.time_base_seconds_per_div, |t| {
                    t.seconds_per_div
                }),
                trigger: TriggerSetup {
                    source: 0,
                    level_volts: 0.0,
                    slope: Slope::Rising,
                    sweep: Sweep::Auto,
                    horizontal_position_percent: 0.0,
                    kind: TriggerKind::Edge,
                },
            },
            probes: [1.0; CHANNELS],
            invert: [false; CHANNELS],
            running: false,
            trigger_fraction: defaults.trigger_fraction,
        };
        model.settle(caps);
        model
    }

    /// Brings a setup kept from an earlier connection in line with the capabilities of the new one.
    pub fn revalidate(&mut self, caps: &Capabilities) {
        let last_range = caps.vertical_ranges.len().saturating_sub(1) as u8;
        let last_time_base = caps.time_bases.len().saturating_sub(1) as u8;
        self.setup.time_base_index = self.setup.time_base_index.min(last_time_base);
        for channel in &mut self.setup.channels {
            channel.range_index = channel.range_index.min(last_range);
        }
        for number in 0..CHANNELS {
            self.clamp_position(caps, number);
        }
        if self.enabled_count() == 0 {
            self.setup.channels[0].enabled = true;
        }
        self.keep_source_enabled();
        self.settle(caps);
    }

    pub fn enabled_count(&self) -> u8 {
        self.setup.channels.iter().filter(|c| c.enabled).count() as u8
    }

    pub fn keep_source_enabled(&mut self) {
        let source = usize::from(self.setup.trigger.source);
        if !self.setup.channels[source].enabled {
            let first = self.setup.channels.iter().position(|c| c.enabled).unwrap_or(0);
            self.setup.trigger.source = first as u8;
        }
    }

    pub fn range<'a>(&self, caps: &'a Capabilities, number: usize) -> Option<&'a VerticalRange> {
        caps.vertical_ranges
            .get(usize::from(self.setup.channels[number].range_index))
    }

    pub fn clamp_position(&mut self, caps: &Capabilities, number: usize) {
        let limit = self.range(caps, number).map_or(0.0, |r| r.max_position_divs);
        let channel = &mut self.setup.channels[number];
        channel.position_divs = channel.position_divs.clamp(-limit, limit);
    }

    pub fn geometry(&self, caps: &Capabilities) -> Geometry {
        let interval = caps.sample_interval_s(self.enabled_count(), self.setup.time_base_index);
        let seconds_per_div = caps
            .time_bases
            .get(usize::from(self.setup.time_base_index))
            .map_or(0.0, |t| t.seconds_per_div);
        geometry(caps.record_len as usize, interval, seconds_per_div)
    }

    /// Recomputes what depends on the geometry; call after every edit.
    pub fn settle(&mut self, caps: &Capabilities) {
        let g = self.geometry(caps);
        self.setup.trigger.horizontal_position_percent =
            record_percent(&g, caps.record_len as usize, self.trigger_fraction);
    }

    pub fn source_range_volts(&self, caps: &Capabilities) -> f64 {
        let source = usize::from(self.setup.trigger.source);
        self.range(caps, source).map_or(1.0, |r| r.volts_per_div)
    }

    pub fn level_divisions(&self, caps: &Capabilities) -> f64 {
        let source = usize::from(self.setup.trigger.source);
        let volts_per_div = self.source_range_volts(caps);
        self.setup.trigger.level_volts / volts_per_div + self.setup.channels[source].position_divs
    }

    pub fn set_level_divisions(&mut self, caps: &Capabilities, divisions: f64) {
        let source = usize::from(self.setup.trigger.source);
        let position = self.setup.channels[source].position_divs;
        self.setup.trigger.level_volts = (divisions - position) * self.source_range_volts(caps);
    }
}
