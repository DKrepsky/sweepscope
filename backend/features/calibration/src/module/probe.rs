//! The setup of one probe: everything the zero measurement needs and nothing the user chose (ADR 0021 item 3).
use sweepscope_kernel::contracts::{
    AcquisitionSetup, Capabilities, ChannelSetup, Coupling, Slope, Sweep, TriggerKind, TriggerSetup,
};

use crate::domain::rules::PROBE_TIME_BASE_SECONDS_PER_DIV;

const CHANNELS: usize = 4;
const TRIGGER_POSITION_PERCENT: f64 = 50.0;

/// All channels on so the converter runs in the four-channel mode of the reference data; `None` without a usable time base.
pub fn probe_setup(capabilities: &Capabilities, range_index: usize, channel: usize) -> Option<AcquisitionSetup> {
    let range_index = u8::try_from(range_index).ok()?;
    let time_base_index = probe_time_base(capabilities)?;
    let channel_setup = ChannelSetup {
        enabled: true,
        range_index,
        position_divs: 0.0,
        coupling: Coupling::Dc,
        bandwidth_limit: false,
    };
    Some(AcquisitionSetup {
        channels: std::array::from_fn::<_, CHANNELS, _>(|_| channel_setup.clone()),
        time_base_index,
        trigger: TriggerSetup {
            source: u8::try_from(channel).ok()?,
            level_volts: 0.0,
            slope: Slope::Rising,
            sweep: Sweep::Auto,
            horizontal_position_percent: TRIGGER_POSITION_PERCENT,
            kind: TriggerKind::Edge,
        },
    })
}

fn probe_time_base(capabilities: &Capabilities) -> Option<u8> {
    let distance = |seconds: f64| (seconds / PROBE_TIME_BASE_SECONDS_PER_DIV).ln().abs();
    capabilities
        .time_bases
        .iter()
        .enumerate()
        .filter(|(_, step)| !step.streaming)
        .min_by(|a, b| distance(a.1.seconds_per_div).total_cmp(&distance(b.1.seconds_per_div)))
        .and_then(|(index, _)| u8::try_from(index).ok())
}

#[cfg(test)]
mod tests {
    use sweepscope_kernel::contracts::{TimeBaseStep, VerticalRange};

    use super::*;

    fn capabilities() -> Capabilities {
        let step = |seconds_per_div: f64, streaming: bool| TimeBaseStep {
            seconds_per_div,
            streaming,
            verified: true,
        };
        Capabilities {
            channel_count: 4,
            record_len: 4096,
            time_bases: vec![step(1e-6, false), step(1e-3, false), step(2e-3, false), step(2.0, true)],
            vertical_ranges: vec![VerticalRange {
                volts_per_div: 0.1,
                verified: true,
                max_position_divs: 4.0,
            }],
            adc_interval_ns: vec![1.0, 2.0, 4.0, 4.0],
            samples_per_div: 250.0,
        }
    }

    #[test]
    fn given_capabilities_when_the_probe_setup_is_built_then_all_channels_are_dc_unlimited_and_at_position_zero() {
        let setup = probe_setup(&capabilities(), 0, 2).unwrap();

        assert!(setup
            .channels
            .iter()
            .all(|c| { c.enabled && c.coupling == Coupling::Dc && !c.bandwidth_limit && c.position_divs == 0.0 }));
    }

    #[test]
    fn given_a_range_and_channel_when_the_probe_setup_is_built_then_one_range_applies_and_the_channel_triggers() {
        let setup = probe_setup(&capabilities(), 7, 3).unwrap();

        assert!(setup.channels.iter().all(|c| c.range_index == 7));
        assert_eq!(setup.trigger.source, 3);
        assert_eq!(setup.trigger.sweep, Sweep::Auto);
        assert_eq!(setup.trigger.level_volts, 0.0);
    }

    #[test]
    fn given_time_bases_when_the_probe_setup_is_built_then_the_two_millisecond_step_is_chosen() {
        assert_eq!(probe_setup(&capabilities(), 0, 0).unwrap().time_base_index, 2);
    }

    #[test]
    fn given_no_time_bases_when_the_probe_setup_is_built_then_there_is_none() {
        let mut caps = capabilities();
        caps.time_bases.clear();

        assert_eq!(probe_setup(&caps, 0, 0), None);
    }
}
