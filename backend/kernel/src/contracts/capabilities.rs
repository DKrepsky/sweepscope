use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../frontend/src/api/generated/")]
pub struct TimeBaseStep {
    pub seconds_per_div: f64,
    /// Roll mode: samples stream instead of being captured as a record.
    pub streaming: bool,
    pub verified: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../frontend/src/api/generated/")]
pub struct VerticalRange {
    pub volts_per_div: f64,
    pub verified: bool,
    /// Largest vertical offset the hardware can reach on this range, in divisions (positive and negative).
    pub max_position_divs: f64,
}

/// What an instrument can do, as data, so the UI never hard-codes a scale table.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../frontend/src/api/generated/")]
pub struct Capabilities {
    pub channel_count: u8,
    pub record_len: u32,
    /// Ordered fast to slow.
    pub time_bases: Vec<TimeBaseStep>,
    pub vertical_ranges: Vec<VerticalRange>,
    /// Index is the enabled channel count minus one.
    pub adc_interval_ns: Vec<f64>,
    pub samples_per_div: f64,
}

impl Capabilities {
    /// `max(adc interval, time base / samples per div)`; an unknown channel count or time base index counts as no limit.
    pub fn sample_interval_s(&self, enabled_channels: u8, time_base_index: u8) -> f64 {
        let adc_s = usize::from(enabled_channels)
            .checked_sub(1)
            .and_then(|i| self.adc_interval_ns.get(i))
            .map_or(0.0, |ns| ns * 1e-9);
        let timebase_s = self
            .time_bases
            .get(usize::from(time_base_index))
            .map_or(0.0, |step| step.seconds_per_div / self.samples_per_div);
        adc_s.max(timebase_s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capabilities() -> Capabilities {
        let seconds = [2e-9, 200e-6, 200e-3, 500e-3];
        Capabilities {
            channel_count: 4,
            record_len: 4096,
            time_bases: seconds
                .iter()
                .map(|&seconds_per_div| TimeBaseStep {
                    seconds_per_div,
                    streaming: false,
                    verified: false,
                })
                .collect(),
            vertical_ranges: vec![],
            adc_interval_ns: vec![1.0, 2.0, 4.0, 4.0],
            samples_per_div: 250.0,
        }
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < expected * 1e-9, "{actual} != {expected}");
    }

    #[test]
    fn given_one_two_and_four_channels_when_the_time_base_is_fastest_then_the_adc_interval_is_used() {
        let caps = capabilities();

        assert_close(caps.sample_interval_s(1, 0), 1e-9);
        assert_close(caps.sample_interval_s(2, 0), 2e-9);
        assert_close(caps.sample_interval_s(4, 0), 4e-9);
    }

    #[test]
    fn given_three_channels_when_the_interval_is_asked_then_it_equals_four_channels() {
        let caps = capabilities();

        assert_close(caps.sample_interval_s(3, 0), caps.sample_interval_s(4, 0));
    }

    #[test]
    fn given_four_channels_when_at_200_us_per_div_then_the_interval_is_800_ns() {
        assert_close(capabilities().sample_interval_s(4, 1), 800e-9);
    }

    #[test]
    fn given_500_ms_per_div_when_the_interval_is_asked_then_it_is_2_ms() {
        assert_close(capabilities().sample_interval_s(4, 3), 2e-3);
    }

    #[test]
    fn given_200_ms_per_div_when_the_interval_is_asked_then_it_is_0_8_ms() {
        assert_close(capabilities().sample_interval_s(4, 2), 0.8e-3);
    }

    #[test]
    fn given_zero_channels_or_an_unknown_time_base_when_the_interval_is_asked_then_the_other_limit_applies() {
        let caps = capabilities();

        assert_close(caps.sample_interval_s(0, 1), 800e-9);
        assert_close(caps.sample_interval_s(4, 200), 4e-9);
    }
}
