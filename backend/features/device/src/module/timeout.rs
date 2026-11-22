use std::time::Duration;

use crate::{AcquisitionSetup, Capabilities};

const MINIMUM: Duration = Duration::from_millis(250);
const RECORD_MARGIN: f64 = 1.5;

/// `max(250 ms, 1.5 x record duration)` (ADR 0007 decision 8): long enough for a full record, short enough to notice a dead device.
pub fn capture_timeout(capabilities: &Capabilities, setup: &AcquisitionSetup) -> Duration {
    let enabled = setup.channels.iter().filter(|channel| channel.enabled).count();
    let interval = capabilities.sample_interval_s(u8::try_from(enabled).unwrap_or(u8::MAX), setup.time_base_index);
    let record = Duration::try_from_secs_f64(interval * f64::from(capabilities.record_len)).unwrap_or(MINIMUM);
    record.mul_f64(RECORD_MARGIN).max(MINIMUM)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChannelSetup, Coupling, Slope, Sweep, TimeBaseStep, TriggerSetup};

    fn capabilities() -> Capabilities {
        let step = |seconds_per_div| TimeBaseStep {
            seconds_per_div,
            streaming: false,
            verified: true,
        };
        Capabilities {
            channel_count: 4,
            record_len: 4096,
            time_bases: vec![step(2e-9), step(0.2), step(1.0)],
            vertical_ranges: vec![],
            adc_interval_ns: vec![1.0, 2.0, 4.0, 4.0],
            samples_per_div: 250.0,
        }
    }

    fn setup(time_base_index: u8) -> AcquisitionSetup {
        let channel = ChannelSetup {
            enabled: true,
            range_index: 0,
            position_divs: 0.0,
            coupling: Coupling::Dc,
            bandwidth_limit: false,
        };
        AcquisitionSetup {
            channels: [channel.clone(), channel.clone(), channel.clone(), channel],
            time_base_index,
            trigger: TriggerSetup {
                source: 0,
                level_volts: 0.0,
                slope: Slope::Rising,
                sweep: Sweep::Auto,
                horizontal_position_percent: 50.0,
            },
        }
    }

    #[test]
    fn given_a_fast_time_base_when_the_timeout_is_computed_then_it_is_the_minimum() {
        assert_eq!(capture_timeout(&capabilities(), &setup(0)), Duration::from_millis(250));
    }

    #[test]
    fn given_a_slow_time_base_when_the_timeout_is_computed_then_it_is_one_and_a_half_records() {
        // 0.2 s/div / 250 per div = 0.8 ms per sample, 4096 samples = 3.2768 s.
        let timeout = capture_timeout(&capabilities(), &setup(1));

        assert!((timeout.as_secs_f64() - 3.2768 * 1.5).abs() < 1e-6, "{timeout:?}");
    }

    #[test]
    fn given_an_unknown_time_base_when_the_timeout_is_computed_then_it_is_the_minimum() {
        assert_eq!(
            capture_timeout(&capabilities(), &setup(200)),
            Duration::from_millis(250)
        );
    }
}
