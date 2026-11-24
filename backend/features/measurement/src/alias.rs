//! Alias warnings per channel: raised above a fifth of the sample rate, cleared below 0.95 of that (hysteresis).
use engineering_format::{format, Quantity};
use sweepscope_kernel::contracts::{Frame, FrameContext};

use crate::messages::{ChannelMeasurements, MeasurementId, Warning, WarningSeverity};

const RAISE_FRACTION: f64 = 0.2;
const CLEAR_FACTOR: f64 = 0.95;
/// The device samples a time base at `seconds_per_div / 250` when that is slower than the converter allows.
const SAMPLES_PER_DIV: f64 = 250.0;
const SAME_INTERVAL: f64 = 0.01;
const MATH_CHANNEL: u8 = 5;

struct Frequency {
    hertz: f64,
    text: String,
}

#[derive(Default)]
pub struct AliasWatch {
    raised: [bool; 4],
}

impl AliasWatch {
    /// The warnings of this frame, ascending by channel; a channel without a valid frequency has none.
    pub fn evaluate(
        &mut self,
        channels: &[ChannelMeasurements],
        frame: &Frame,
        context: &FrameContext,
    ) -> Vec<Warning> {
        let sample_rate = 1.0 / frame.sample_interval_s;
        let cause = cause_sentence(frame, context);
        let mut now_raised = [false; 4];
        let mut warnings = Vec::new();
        for channel in channels.iter().filter(|c| c.channel != MATH_CHANNEL) {
            let slot = usize::from(channel.channel - 1);
            let Some(frequency) = frequency_of(channel) else {
                continue;
            };
            now_raised[slot] = self.stays_or_becomes_raised(slot, frequency.hertz, sample_rate);
            if now_raised[slot] {
                warnings.push(warning(channel.channel, &frequency.text, sample_rate, cause));
            }
        }
        self.raised = now_raised;
        warnings
    }

    fn stays_or_becomes_raised(&self, slot: usize, hertz: f64, sample_rate: f64) -> bool {
        let limit = sample_rate * RAISE_FRACTION;
        if self.raised[slot] {
            hertz > limit * CLEAR_FACTOR
        } else {
            hertz > limit
        }
    }
}

fn frequency_of(channel: &ChannelMeasurements) -> Option<Frequency> {
    let value = channel
        .measurements
        .iter()
        .find(|m| m.id == MeasurementId::Frequency)?
        .value
        .as_ref()?;
    Some(Frequency {
        hertz: value.value,
        text: format!("{} {}", value.text, value.unit),
    })
}

fn cause_sentence(frame: &Frame, context: &FrameContext) -> &'static str {
    let time_base_interval = context.seconds_per_div / SAMPLES_PER_DIV;
    let limited_by_time_base =
        (frame.sample_interval_s - time_base_interval).abs() <= time_base_interval * SAME_INTERVAL;
    if limited_by_time_base {
        " Use a faster time base to raise the sample rate."
    } else if frame.channels.iter().flatten().count() > 1 {
        " Enable fewer channels to raise the sample rate."
    } else {
        ""
    }
}

fn warning(channel: u8, frequency_text: &str, sample_rate: f64, cause: &str) -> Warning {
    let rate = format(sample_rate, Quantity::SamplesPerSecond);
    Warning {
        id: format!("alias-ch{channel}"),
        severity: WarningSeverity::Warning,
        channel: Some(channel),
        short: format!("Alias CH{channel}"),
        message: format!(
            "Possible aliasing on CH{channel}. Its frequency ({frequency_text}) is more than one fifth of the sample rate ({} {}), so the display may show a wrong shape or frequency.{cause}",
            rate.text, rate.unit
        ),
    }
}

#[cfg(test)]
mod tests {
    use sweepscope_kernel::contracts::{ChannelData, FormattedValue, TriggerOutcome};

    use super::*;
    use crate::messages::{InvalidReason, Measurement};
    use crate::texts;

    const FS: f64 = 250e6;
    const CAUSE_TIME_BASE: &str = " Use a faster time base to raise the sample rate.";
    const CAUSE_CHANNELS: &str = " Enable fewer channels to raise the sample rate.";

    fn data() -> Option<ChannelData> {
        Some(ChannelData {
            codes: vec![0; 4],
            volts_per_div: 1.0,
            zero_code: 0.0,
        })
    }

    fn frame(rate: f64, channels: usize) -> Frame {
        let mut slots = [None, None, None, None];
        slots.iter_mut().take(channels).for_each(|slot| *slot = data());
        Frame {
            channels: slots,
            counts_per_div: 100.0,
            sample_interval_s: 1.0 / rate,
            trigger: TriggerOutcome::None,
        }
    }

    fn context(seconds_per_div: f64) -> FrameContext {
        FrameContext {
            setup_revision: 1,
            window_start: 0,
            window_len: 4,
            invert: [false; 4],
            probe: [1.0; 4],
            range_verified: [true; 4],
            seconds_per_div,
        }
    }

    fn channel(number: u8, hertz: Option<f64>) -> ChannelMeasurements {
        let value = hertz.map(|hertz| FormattedValue {
            value: hertz,
            text: format!("{:.1}", hertz / 1e6),
            unit: "MHz".into(),
        });
        ChannelMeasurements {
            channel: number,
            clipped: false,
            unverified: None,
            measurements: vec![Measurement {
                id: MeasurementId::Frequency,
                label: "Frequency".into(),
                invalid: value.is_none().then(|| texts::invalid(InvalidReason::NoSignal)),
                value,
            }],
        }
    }

    fn evaluate(watch: &mut AliasWatch, mhz: f64) -> Vec<Warning> {
        watch.evaluate(&[channel(1, Some(mhz * 1e6))], &frame(FS, 1), &context(100e-9))
    }

    #[test]
    fn given_62_mhz_at_250_ms_per_s_when_evaluated_then_alias_ch1_is_raised_with_the_exact_message() {
        let mut watch = AliasWatch::default();

        let warnings = evaluate(&mut watch, 62.0);

        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].id, "alias-ch1");
        assert_eq!(warnings[0].short, "Alias CH1");
        assert_eq!(warnings[0].channel, Some(1));
        assert_eq!(
            warnings[0].message,
            "Possible aliasing on CH1. Its frequency (62.0 MHz) is more than one fifth of the sample rate (250 MS/s), so the display may show a wrong shape or frequency."
        );
    }

    #[test]
    fn given_a_raised_alias_when_49_9_mhz_follows_then_it_stays() {
        let mut watch = AliasWatch::default();
        evaluate(&mut watch, 62.0);

        let warnings = evaluate(&mut watch, 49.9);

        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn given_a_raised_alias_when_47_mhz_follows_then_it_clears() {
        let mut watch = AliasWatch::default();
        evaluate(&mut watch, 62.0);

        let warnings = evaluate(&mut watch, 47.0);

        assert!(warnings.is_empty());
    }

    #[test]
    fn given_no_alias_when_49_9_mhz_is_measured_then_it_is_not_raised() {
        let mut watch = AliasWatch::default();

        let warnings = evaluate(&mut watch, 49.9);

        assert!(warnings.is_empty());
    }

    #[test]
    fn given_an_invalid_frequency_when_evaluated_then_there_is_no_warning() {
        let mut watch = AliasWatch::default();

        let warnings = watch.evaluate(&[channel(1, None)], &frame(FS, 1), &context(100e-9));

        assert!(warnings.is_empty());
    }

    #[test]
    fn given_1_khz_at_100_ms_per_div_when_evaluated_then_the_time_base_sentence_is_appended() {
        let mut watch = AliasWatch::default();

        let warnings = watch.evaluate(&[channel(1, Some(1e3))], &frame(2.5e3, 1), &context(0.1));

        assert_eq!(warnings.len(), 1);
        assert!(
            warnings[0].message.ends_with(CAUSE_TIME_BASE),
            "{}",
            warnings[0].message
        );
    }

    #[test]
    fn given_two_channels_at_the_converter_limit_when_alias_is_raised_then_the_fewer_channels_sentence_is_appended() {
        let mut watch = AliasWatch::default();
        let channels = [channel(1, Some(62e6)), channel(2, Some(1e6))];

        let warnings = watch.evaluate(&channels, &frame(FS, 2), &context(100e-9));

        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.ends_with(CAUSE_CHANNELS));
    }

    #[test]
    fn given_alias_on_two_channels_when_one_falls_below_then_only_that_one_clears() {
        let mut watch = AliasWatch::default();
        let both = [channel(1, Some(62e6)), channel(2, Some(62e6))];
        watch.evaluate(&both, &frame(FS, 2), &context(100e-9));

        let after = [channel(1, Some(62e6)), channel(2, Some(10e6))];
        let warnings = watch.evaluate(&after, &frame(FS, 2), &context(100e-9));

        let ids: Vec<&str> = warnings.iter().map(|w| w.id.as_str()).collect();
        assert_eq!(ids, vec!["alias-ch1"]);
    }

    #[test]
    fn given_a_fast_math_trace_when_evaluated_then_the_math_column_never_raises_alias() {
        let mut watch = AliasWatch::default();

        let warnings = watch.evaluate(&[channel(5, Some(62e6))], &frame(FS, 1), &context(100e-9));

        assert!(warnings.is_empty());
    }
}
