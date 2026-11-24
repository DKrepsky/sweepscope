//! Pure translations between the device-neutral contracts and the driver's types (ADR 0012 item 3).

use hantek6xx4_lib::protocol::channel_mode;
use hantek6xx4_lib::{
    counts_per_div, zero_code_for_position, Channel, ChannelConfig, Config, Coupling as DriverCoupling,
    Error as DriverError, Frame as DriverFrame, Slope as DriverSlope, Sweep as DriverSweep, TimeBase, TriggerConfig,
    TriggerKind, VoltsPerDiv, MAX_CHANNELS,
};

use crate::profile::RECORD_LEN;
use crate::{AcquisitionSetup, ChannelData, ChannelSetup, Coupling, DeviceError, Frame, Slope, Sweep, TriggerOutcome};

/// ADC code of the screen centre (zero volts at position 0).
const CENTRE_CODE: f64 = 128.0;
/// Highest trigger level code the driver accepts.
const MAX_TRIGGER_CODE: f64 = 228.0;

pub fn config_from_setup(setup: &AcquisitionSetup) -> Result<Config, DeviceError> {
    let timebase = TimeBase::from_index(setup.time_base_index).map_err(device_error)?;
    let enabled = setup.channels.iter().filter(|channel| channel.enabled).count();
    let mode = channel_mode(enabled, timebase);
    let mut channels = [ChannelConfig::default(); MAX_CHANNELS];
    for (slot, setup_channel) in channels.iter_mut().zip(&setup.channels) {
        *slot = channel_config(setup_channel, mode)?;
    }
    let trigger = trigger_config(setup, &channels, mode)?;
    Ok(Config {
        channels,
        timebase,
        trigger,
        buffer_len: RECORD_LEN,
    })
}

fn channel_config(setup: &ChannelSetup, mode: u8) -> Result<ChannelConfig, DeviceError> {
    if !setup.enabled {
        return Ok(ChannelConfig {
            enabled: false,
            ..ChannelConfig::default()
        });
    }
    if !setup.position_divs.is_finite() {
        return Err(DeviceError::InvalidSetting(
            "position must be a finite number".to_owned(),
        ));
    }
    let volts_per_div = VoltsPerDiv::from_index(setup.range_index).map_err(device_error)?;
    Ok(ChannelConfig {
        enabled: true,
        volts_per_div,
        coupling: match setup.coupling {
            Coupling::Ac => DriverCoupling::Ac,
            Coupling::Dc => DriverCoupling::Dc,
        },
        bandwidth_limit: setup.bandwidth_limit,
        position: position_byte(setup.position_divs, volts_per_div, mode),
    })
}

fn trigger_config(
    setup: &AcquisitionSetup,
    channels: &[ChannelConfig; MAX_CHANNELS],
    mode: u8,
) -> Result<TriggerConfig, DeviceError> {
    let trigger = &setup.trigger;
    if !trigger.level_volts.is_finite() {
        return Err(DeviceError::InvalidSetting(
            "trigger level must be a finite number".to_owned(),
        ));
    }
    let source = Channel::from_index(usize::from(trigger.source)).map_err(device_error)?;
    Ok(TriggerConfig {
        source,
        kind: TriggerKind::Edge,
        slope: match trigger.slope {
            Slope::Rising => DriverSlope::Rising,
            Slope::Falling => DriverSlope::Falling,
        },
        sweep: sweep(trigger.sweep),
        level: trigger_code(trigger.level_volts, &channels[source.index()], mode),
        horizontal_position_percent: trigger.horizontal_position_percent,
    })
}

fn sweep(sweep: Sweep) -> DriverSweep {
    match sweep {
        Sweep::Auto => DriverSweep::Auto,
        Sweep::Normal => DriverSweep::Normal,
    }
}

/// Inverts `zero_code_for_position`: the zero-volt line sits `position_divs` above the screen centre, i.e. at
/// `centre + divs * counts_per_div`. Clamped to the byte, which is what limits the reachable span.
pub fn position_byte(position_divs: f64, volts_per_div: VoltsPerDiv, mode: u8) -> u8 {
    let wanted_zero = CENTRE_CODE + position_divs * counts_per_div(mode);
    let at_zero = zero_code_for_position(0, volts_per_div, mode);
    let at_full = zero_code_for_position(255, volts_per_div, mode);
    let byte = (wanted_zero - at_zero) * 255.0 / (at_full - at_zero);
    byte.round().clamp(0.0, 255.0) as u8
}

/// Trigger level in volts as an ADC code, using the zero model of the channel as configured (protocol.md 9).
pub fn trigger_code(level_volts: f64, source: &ChannelConfig, mode: u8) -> u8 {
    let zero = zero_code_for_position(source.position, source.volts_per_div, mode);
    let counts = level_volts / source.volts_per_div.volts() * counts_per_div(mode);
    (zero + counts).round().clamp(0.0, MAX_TRIGGER_CODE) as u8
}

pub fn capture_frame(frame: &DriverFrame) -> Frame {
    let trigger = match (frame.triggered, frame.trigger_sample) {
        (true, Some(sample_index)) => TriggerOutcome::Triggered { sample_index },
        _ => TriggerOutcome::Auto,
    };
    kernel_frame(frame, trigger)
}

pub fn roll_frame(frame: &DriverFrame) -> Frame {
    kernel_frame(frame, TriggerOutcome::None)
}

fn kernel_frame(frame: &DriverFrame, trigger: TriggerOutcome) -> Frame {
    let channel = |index: usize| {
        let codes = &frame.samples[index];
        (!codes.is_empty()).then(|| ChannelData {
            codes: codes.clone(),
            volts_per_div: frame.volts_per_div[index],
            zero_code: frame.zero_code[index],
        })
    };
    Frame {
        channels: [channel(0), channel(1), channel(2), channel(3)],
        counts_per_div: frame.counts_per_div,
        sample_interval_s: frame.sample_interval_ns * 1e-9,
        trigger,
    }
}

pub fn device_error(error: DriverError) -> DeviceError {
    match error {
        DriverError::Usb(rusb::Error::Access) => DeviceError::PermissionDenied,
        DriverError::Disconnected => DeviceError::Disconnected,
        DriverError::NotFound => DeviceError::NotFound,
        DriverError::Timeout(_) => DeviceError::Timeout,
        DriverError::Overrun => DeviceError::Overrun,
        DriverError::BadCalibration => DeviceError::CalibrationInvalid,
        DriverError::InvalidParameter(reason) => DeviceError::InvalidSetting(reason),
        other => DeviceError::Io(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::capabilities;
    use crate::TriggerSetup;

    fn setup() -> AcquisitionSetup {
        let channel = || ChannelSetup {
            enabled: true,
            range_index: 8,
            position_divs: 0.0,
            coupling: Coupling::Dc,
            bandwidth_limit: false,
        };
        AcquisitionSetup {
            channels: [channel(), channel(), channel(), channel()],
            time_base_index: 15,
            trigger: TriggerSetup {
                source: 0,
                level_volts: 0.0,
                slope: Slope::Rising,
                sweep: Sweep::Auto,
                horizontal_position_percent: 50.0,
                kind: sweepscope_kernel::contracts::TriggerKind::Edge,
            },
        }
    }

    fn range(index: u8) -> VoltsPerDiv {
        VoltsPerDiv::from_index(index).unwrap()
    }

    fn close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() <= expected.abs() * 1e-12,
            "{actual} != {expected}"
        );
    }

    fn driver_frame() -> DriverFrame {
        DriverFrame {
            samples: [vec![1, 2, 3], vec![], vec![4, 5, 6], vec![]],
            volts_per_div: [1.0, 0.0, 0.5, 0.0],
            zero_code: [128.0, 0.0, 120.0, 0.0],
            counts_per_div: 24.15,
            sample_interval_ns: 800.0,
            triggered: false,
            trigger_sample: None,
        }
    }

    #[test]
    fn given_each_range_index_when_compared_with_the_driver_then_the_volts_are_the_same() {
        let ranges = capabilities().vertical_ranges;

        assert_eq!(ranges.len(), usize::from(VoltsPerDiv::COUNT));
        for (index, entry) in ranges.iter().enumerate() {
            close(entry.volts_per_div, range(index as u8).volts());
        }
    }

    #[test]
    fn given_each_time_base_index_when_compared_with_the_driver_then_the_seconds_are_the_same() {
        let bases = capabilities().time_bases;

        assert_eq!(bases.len(), usize::from(TimeBase::COUNT));
        for (index, base) in bases.iter().enumerate() {
            let driver = TimeBase::from_index(index as u8).unwrap();
            close(base.seconds_per_div, driver.seconds_per_div());
        }
    }

    #[test]
    fn given_a_setup_when_converted_then_every_field_maps_to_the_driver_config() {
        let mut setup = setup();
        setup.channels[1].enabled = false;
        setup.channels[2].coupling = Coupling::Ac;
        setup.channels[3].bandwidth_limit = true;
        setup.trigger.slope = Slope::Falling;
        setup.trigger.sweep = Sweep::Normal;
        setup.trigger.source = 2;
        setup.trigger.horizontal_position_percent = 25.0;

        let config = config_from_setup(&setup).unwrap();

        assert_eq!(config.timebase.index(), 15);
        assert_eq!(config.buffer_len, 4096);
        assert!(!config.channels[1].enabled);
        assert_eq!(config.channels[2].coupling, DriverCoupling::Ac);
        assert!(config.channels[3].bandwidth_limit);
        assert_eq!(config.channels[0].volts_per_div.index(), 8);
        assert_eq!(config.trigger.source, Channel::Ch3);
        assert_eq!(config.trigger.slope, DriverSlope::Falling);
        assert_eq!(config.trigger.sweep, DriverSweep::Normal);
        assert_eq!(config.trigger.horizontal_position_percent, 25.0);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn given_an_out_of_table_index_when_converted_then_the_setting_is_invalid() {
        let mut bad_range = setup();
        bad_range.channels[0].range_index = 12;
        let mut bad_base = setup();
        bad_base.time_base_index = 36;

        assert!(matches!(
            config_from_setup(&bad_range),
            Err(DeviceError::InvalidSetting(_))
        ));
        assert!(matches!(
            config_from_setup(&bad_base),
            Err(DeviceError::InvalidSetting(_))
        ));
    }

    #[test]
    fn given_a_non_finite_position_or_level_when_converted_then_the_setting_is_invalid() {
        let mut position = setup();
        position.channels[0].position_divs = f64::NAN;
        let mut level = setup();
        level.trigger.level_volts = f64::INFINITY;

        assert!(matches!(
            config_from_setup(&position),
            Err(DeviceError::InvalidSetting(_))
        ));
        assert!(matches!(config_from_setup(&level), Err(DeviceError::InvalidSetting(_))));
    }

    #[test]
    fn given_a_disabled_trigger_source_when_converted_then_the_driver_refuses_the_config() {
        let mut setup = setup();
        setup.channels[0].enabled = false;

        let config = config_from_setup(&setup).unwrap();

        assert!(config.validate().is_err());
    }

    #[test]
    fn given_a_disabled_channel_with_a_bad_range_when_converted_then_it_is_ignored() {
        let mut setup = setup();
        setup.channels[3].enabled = false;
        setup.channels[3].range_index = 99;

        assert!(config_from_setup(&setup).is_ok());
    }

    #[test]
    fn given_plus_2_divisions_in_four_channel_mode_when_converted_then_the_zero_code_reproduces_it() {
        let byte = position_byte(2.0, range(8), 4);

        let zero = zero_code_for_position(byte, range(8), 4);
        let divisions = (zero - 128.0) / counts_per_div(4);
        let one_byte_in_divisions = (224.0 - 31.0) / 255.0 / counts_per_div(4);
        assert!((divisions - 2.0).abs() <= one_byte_in_divisions, "{divisions}");
    }

    #[test]
    fn given_plus_4_divisions_on_a_narrow_range_when_converted_then_it_clamps_to_the_published_limit() {
        let published = capabilities().vertical_ranges[7].max_position_divs;

        let byte = position_byte(4.0, range(7), 4);

        assert_eq!(byte, 255);
        let reached = (zero_code_for_position(byte, range(7), 4) - 128.0) / counts_per_div(4);
        assert!((reached - published).abs() < 0.1, "{reached} vs {published}");
    }

    #[test]
    fn given_beyond_minus_4_divisions_on_a_wide_range_when_converted_then_the_byte_is_zero() {
        assert_eq!(position_byte(-4.1, range(8), 4), 0);
        assert_eq!(position_byte(-9.0, range(8), 4), 0);
    }

    #[test]
    fn given_position_zero_when_converted_then_the_zero_line_is_at_the_centre_within_one_byte() {
        for mode in [1, 2, 4] {
            let zero = zero_code_for_position(position_byte(0.0, range(8), mode), range(8), mode);

            assert!((zero - 128.0).abs() < 1.0, "mode {mode}: {zero}");
        }
    }

    #[test]
    fn given_a_level_of_0_v_at_position_0_when_converted_then_the_code_equals_the_zero_code() {
        let config = config_from_setup(&setup()).unwrap();
        let zero = zero_code_for_position(config.channels[0].position, range(8), 4);

        assert_eq!(f64::from(config.trigger.level), zero.round());
    }

    #[test]
    fn given_a_level_of_one_division_when_converted_then_the_code_is_one_division_above_zero() {
        let mut setup = setup();
        setup.trigger.level_volts = 1.0;

        let config = config_from_setup(&setup).unwrap();

        let zero = zero_code_for_position(config.channels[0].position, range(8), 4);
        assert_eq!(f64::from(config.trigger.level), (zero + counts_per_div(4)).round());
    }

    #[test]
    fn given_a_level_above_or_below_the_window_when_converted_then_the_code_clamps() {
        let mut high = setup();
        high.trigger.level_volts = 100.0;
        let mut low = setup();
        low.trigger.level_volts = -100.0;

        assert_eq!(config_from_setup(&high).unwrap().trigger.level, 228);
        assert_eq!(config_from_setup(&low).unwrap().trigger.level, 0);
    }

    #[test]
    fn given_a_triggered_driver_frame_when_converted_then_the_outcome_carries_the_trigger_sample() {
        let mut frame = driver_frame();
        frame.triggered = true;
        frame.trigger_sample = Some(2048);

        let converted = capture_frame(&frame);

        assert_eq!(converted.trigger, TriggerOutcome::Triggered { sample_index: 2048 });
    }

    #[test]
    fn given_an_untriggered_driver_frame_when_converted_then_the_outcome_is_auto() {
        assert_eq!(capture_frame(&driver_frame()).trigger, TriggerOutcome::Auto);
    }

    #[test]
    fn given_a_driver_frame_when_converted_then_channels_scale_and_interval_are_kept() {
        let converted = capture_frame(&driver_frame());

        assert!(converted.channels[1].is_none() && converted.channels[3].is_none());
        let first = converted.channels[0].as_ref().unwrap();
        assert_eq!(first.codes, vec![1, 2, 3]);
        assert_eq!((first.volts_per_div, first.zero_code), (1.0, 128.0));
        assert_eq!(converted.channels[2].as_ref().unwrap().volts_per_div, 0.5);
        assert_eq!(converted.counts_per_div, 24.15);
        close(converted.sample_interval_s, 800e-9);
    }

    #[test]
    fn given_a_roll_chunk_when_converted_then_it_has_no_trigger_and_may_be_empty() {
        let mut chunk = driver_frame();
        chunk.samples = Default::default();

        let converted = roll_frame(&chunk);

        assert_eq!(converted.trigger, TriggerOutcome::None);
        assert!(converted.channels.iter().all(Option::is_none));
    }

    #[test]
    fn given_each_driver_error_when_converted_then_it_becomes_the_device_error_of_adr_0012() {
        let cases = [
            (DriverError::Usb(rusb::Error::Access), DeviceError::PermissionDenied),
            (DriverError::Disconnected, DeviceError::Disconnected),
            (DriverError::NotFound, DeviceError::NotFound),
            (DriverError::Timeout("capture"), DeviceError::Timeout),
            (DriverError::Overrun, DeviceError::Overrun),
            (DriverError::BadCalibration, DeviceError::CalibrationInvalid),
            (
                DriverError::InvalidParameter("x".into()),
                DeviceError::InvalidSetting("x".into()),
            ),
        ];
        for (driver, expected) in cases {
            assert_eq!(device_error(driver), expected);
        }
    }

    #[test]
    fn given_other_usb_or_protocol_errors_when_converted_then_they_are_io_errors() {
        assert!(matches!(
            device_error(DriverError::Usb(rusb::Error::Pipe)),
            DeviceError::Io(_)
        ));
        assert!(matches!(
            device_error(DriverError::Protocol("short".into())),
            DeviceError::Io(_)
        ));
    }
}
