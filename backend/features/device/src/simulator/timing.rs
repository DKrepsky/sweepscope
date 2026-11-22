//! Timing model of the simulated scope.
//!
//! Sample interval and the roll buffer span follow the driver (`protocol::sample_interval_ns`, 64 KiB
//! buffer, overrun at 95 % of the span). The dead time between two frames is an assumption:
//! the poll interval (20 ms) comes from the driver's poll loop, but the readback (15 ms) and the
//! re-arm (5 ms) are UNVERIFIED guesses until measured on hardware.

use std::time::Duration;

pub const POLL_INTERVAL: Duration = Duration::from_millis(20);
/// UNVERIFIED: assumed time to read a frame back over USB.
pub const READBACK: Duration = Duration::from_millis(15);
/// UNVERIFIED: assumed time to re-arm the trigger after a readback.
pub const REARM: Duration = Duration::from_millis(5);

pub const ROLL_BUFFER_BYTES: f64 = 65_536.0;
pub const ROLL_OVERRUN_FRACTION: f64 = 0.95;

/// Reference formula that the tests hold `Capabilities::sample_interval_s` against.
#[cfg(test)]
/// Seconds between samples: `max(adc interval, seconds per div / samples per div)`.
pub fn sample_interval(adc_interval_ns: f64, seconds_per_div: f64, samples_per_div: f64) -> f64 {
    (adc_interval_ns * 1e-9).max(seconds_per_div / samples_per_div)
}

/// Time to acquire `record_len` samples.
pub fn record_duration(sample_interval_s: f64, record_len: u32) -> Duration {
    Duration::from_secs_f64(sample_interval_s * f64::from(record_len))
}

/// Clock time one `capture` takes: acquisition rounded up to whole poll intervals, plus readback and re-arm.
pub fn frame_cycle(record: Duration) -> Duration {
    let polls = record.as_nanos().div_ceil(POLL_INTERVAL.as_nanos()).max(1);
    let polled = POLL_INTERVAL * u32::try_from(polls).unwrap_or(u32::MAX);
    polled + READBACK + REARM
}

/// How long the roll buffer takes to fill: `65536 / stride x interval`. `stride` is the ADC channel count (1, 2 or 4).
pub fn roll_span(stride: u8, sample_interval_s: f64) -> Duration {
    Duration::from_secs_f64(ROLL_BUFFER_BYTES / f64::from(stride) * sample_interval_s)
}

/// True once data may have been overwritten: elapsed time since the last poll reached 95 % of the span.
pub fn roll_overruns(elapsed: Duration, span: Duration) -> bool {
    elapsed.as_secs_f64() >= span.as_secs_f64() * ROLL_OVERRUN_FRACTION
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADC_NS: [f64; 4] = [1.0, 2.0, 4.0, 4.0];
    const SAMPLES_PER_DIV: f64 = 250.0;

    fn interval(channels: usize, seconds_per_div: f64) -> f64 {
        sample_interval(ADC_NS[channels - 1], seconds_per_div, SAMPLES_PER_DIV)
    }

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < a.abs() * 1e-9
    }

    #[test]
    fn given_four_channels_at_200_us_per_div_when_interval_is_asked_then_it_is_800_ns() {
        assert!(near(interval(4, 200e-6), 800e-9));
    }

    #[test]
    fn given_500_ms_per_div_when_interval_is_asked_then_it_is_2_ms() {
        assert!(near(interval(4, 0.5), 2e-3));
    }

    #[test]
    fn given_200_ms_per_div_when_interval_is_asked_then_it_is_0_8_ms() {
        assert!(near(interval(4, 0.2), 0.8e-3));
    }

    #[test]
    fn given_one_two_and_four_channels_at_2_ns_per_div_when_interval_is_asked_then_it_is_the_adc_interval() {
        assert!(near(interval(1, 2e-9), 1e-9));
        assert!(near(interval(2, 2e-9), 2e-9));
        assert!(near(interval(3, 2e-9), 4e-9));
        assert!(near(interval(4, 2e-9), 4e-9));
    }

    #[test]
    fn given_a_record_of_4096_samples_when_duration_is_asked_then_it_is_samples_times_interval() {
        assert_eq!(record_duration(800e-9, 4096), Duration::from_nanos(3_276_800));
    }

    #[test]
    fn given_a_short_acquisition_when_the_cycle_is_asked_then_it_is_one_poll_plus_dead_time() {
        assert_eq!(frame_cycle(Duration::from_millis(3)), Duration::from_millis(40));
    }

    #[test]
    fn given_an_instant_acquisition_when_the_cycle_is_asked_then_it_is_still_one_poll_plus_dead_time() {
        assert_eq!(frame_cycle(Duration::ZERO), Duration::from_millis(40));
    }

    #[test]
    fn given_an_acquisition_of_exactly_two_polls_when_the_cycle_is_asked_then_it_is_not_rounded_further() {
        assert_eq!(frame_cycle(Duration::from_millis(40)), Duration::from_millis(60));
    }

    #[test]
    fn given_an_acquisition_just_over_a_poll_when_the_cycle_is_asked_then_it_rounds_up() {
        assert_eq!(frame_cycle(Duration::from_millis(21)), Duration::from_millis(60));
    }

    #[test]
    fn given_four_channels_at_500_ms_per_div_when_span_is_asked_then_it_is_32_8_seconds() {
        let span = roll_span(4, 2e-3);
        assert!((span.as_secs_f64() - 32.768).abs() < 1e-9);
    }

    #[test]
    fn given_one_channel_at_2_ms_when_span_is_asked_then_it_is_131_seconds() {
        assert!((roll_span(1, 2e-3).as_secs_f64() - 131.072).abs() < 1e-9);
    }

    #[test]
    fn given_elapsed_time_at_95_percent_of_the_span_when_checked_then_it_overruns() {
        let span = Duration::from_secs(100);
        assert!(!roll_overruns(Duration::from_millis(94_999), span));
        assert!(roll_overruns(Duration::from_secs(95), span));
    }
}
