use std::sync::Arc;
use std::time::Duration;

use sweepscope_kernel::contracts::{ChannelData, Frame, FrameCaptured, FrameContext, TriggerOutcome};
use sweepscope_kernel::testing::FakeClock;
use sweepscope_kernel::Bus;
use sweepscope_measurement::{
    ChannelMeasurements, GetMeasurements, InvalidReason, MeasurementDeps, MeasurementId, MeasurementModule,
    MeasurementsChanged,
};

const WAIT: Duration = Duration::from_secs(2);
const RECORD: usize = 4096;
const COUNTS_PER_DIV: f64 = 100.0;
const ZERO_CODE: f64 = 30.0;

/// A calibrator-like wave of 0 to 1.91 V (at 1 V/div and 100 counts/div): `edge_s` is the 10 to 90 % time.
struct Wave {
    interval_s: f64,
    period_s: f64,
    edge_s: f64,
}

impl Wave {
    fn level(&self, t: f64) -> f64 {
        let ramp = self.edge_s / 0.8;
        let phase = t % self.period_s;
        let half = self.period_s / 2.0;
        let unit = if phase < ramp {
            phase / ramp
        } else if phase < half {
            1.0
        } else if phase < half + ramp {
            1.0 - (phase - half) / ramp
        } else {
            0.0
        };
        unit * 1.91
    }

    fn codes(&self, offset_s: f64) -> Vec<u8> {
        (0..RECORD)
            .map(|i| {
                let volts = self.level(offset_s + i as f64 * self.interval_s);
                (ZERO_CODE + volts * COUNTS_PER_DIV).round() as u8
            })
            .collect()
    }
}

fn record_interval(seconds_per_div: f64) -> f64 {
    seconds_per_div * 16.4 / RECORD as f64
}

fn calibrator_at(seconds_per_div: f64) -> Wave {
    Wave {
        interval_s: record_interval(seconds_per_div),
        period_s: 1e-3,
        edge_s: 250e-9,
    }
}

fn channel(codes: Vec<u8>) -> ChannelData {
    ChannelData {
        codes,
        volts_per_div: 1.0,
        zero_code: ZERO_CODE,
    }
}

fn frame_of(channels: [Option<ChannelData>; 4], interval_s: f64) -> Frame {
    Frame {
        channels,
        counts_per_div: COUNTS_PER_DIV,
        sample_interval_s: interval_s,
        trigger: TriggerOutcome::None,
    }
}

fn context(seconds_per_div: f64) -> FrameContext {
    FrameContext {
        setup_revision: 1,
        window_start: 0,
        window_len: RECORD,
        invert: [false; 4],
        probe: [1.0; 4],
        range_verified: [true; 4],
        seconds_per_div,
    }
}

fn captured(wave: &Wave, seconds_per_div: f64, offset_s: f64) -> FrameCaptured {
    let channels = [Some(channel(wave.codes(offset_s))), None, None, None];
    FrameCaptured {
        frame: Arc::new(frame_of(channels, wave.interval_s)),
        context: context(seconds_per_div),
        derived: None,
    }
}

fn start(bus: &Bus) -> (MeasurementModule, Arc<FakeClock>) {
    let clock = Arc::new(FakeClock::default());
    let module = MeasurementModule::new(MeasurementDeps {
        bus: bus.clone(),
        clock: clock.clone(),
    })
    .unwrap();
    (module, clock)
}

fn first_channel(bus: &Bus, changes: &sweepscope_kernel::Subscription<MeasurementsChanged>) -> ChannelMeasurements {
    let state = changes.recv_timeout(WAIT).expect("measurements changed").0;
    assert_eq!(bus.request(GetMeasurements).unwrap(), state);
    state.channels.into_iter().next().expect("a channel")
}

fn cell(channel: &ChannelMeasurements, id: MeasurementId) -> (Option<String>, Option<InvalidReason>) {
    let m = channel.measurements.iter().find(|m| m.id == id).unwrap();
    let text = m.value.as_ref().map(|v| format!("{} {}", v.text, v.unit));
    (text, m.invalid.as_ref().map(|i| i.reason))
}

#[test]
fn given_calibrator_frames_at_200_us_per_div_when_measured_then_frequency_and_vpp_read_the_calibrator() {
    let bus = Bus::new();
    let changes = bus.subscribe::<MeasurementsChanged>();
    let (_module, _clock) = start(&bus);

    bus.publish(captured(&calibrator_at(200e-6), 200e-6, 0.1e-3));

    let channel = first_channel(&bus, &changes);
    assert_eq!(channel.channel, 1);
    assert_eq!(channel.measurements.len(), 12);
    assert_eq!(cell(&channel, MeasurementId::Frequency).0.as_deref(), Some("1.000 kHz"));
    assert_eq!(cell(&channel, MeasurementId::Period).0.as_deref(), Some("1.000 ms"));
    assert_eq!(cell(&channel, MeasurementId::Vpp).0.as_deref(), Some("1.91 V"));
}

#[test]
fn given_calibrator_frames_at_200_us_per_div_when_measured_then_rise_time_is_edge_too_fast() {
    let bus = Bus::new();
    let changes = bus.subscribe::<MeasurementsChanged>();
    let (_module, _clock) = start(&bus);

    bus.publish(captured(&calibrator_at(200e-6), 200e-6, 0.1e-3));

    let channel = first_channel(&bus, &changes);
    assert_eq!(
        cell(&channel, MeasurementId::RiseTime),
        (None, Some(InvalidReason::EdgeTooFast))
    );
}

#[test]
fn given_20_us_per_div_with_one_channel_when_measured_then_rise_time_is_about_250_ns() {
    let bus = Bus::new();
    let changes = bus.subscribe::<MeasurementsChanged>();
    let (_module, _clock) = start(&bus);

    bus.publish(captured(&calibrator_at(20e-6), 20e-6, 1e-3 - 100e-6));

    let channel = first_channel(&bus, &changes);
    let rise = channel
        .measurements
        .iter()
        .find(|m| m.id == MeasurementId::RiseTime)
        .and_then(|m| m.value.as_ref())
        .expect("rise time");
    assert!((rise.value - 250e-9).abs() < 25e-9, "{}", rise.value);
}

#[test]
fn given_2_ns_per_div_when_measured_then_frequency_is_fewer_than_one_period() {
    let bus = Bus::new();
    let changes = bus.subscribe::<MeasurementsChanged>();
    let (_module, _clock) = start(&bus);

    bus.publish(captured(&calibrator_at(2e-9), 2e-9, 1e-3 - 10e-9));

    let channel = first_channel(&bus, &changes);
    assert_eq!(
        cell(&channel, MeasurementId::Frequency),
        (None, Some(InvalidReason::FewerThanOnePeriod))
    );
}

#[test]
fn given_a_record_with_a_rail_code_when_measured_then_amplitudes_are_clipped_and_frequency_stays_valid() {
    let bus = Bus::new();
    let changes = bus.subscribe::<MeasurementsChanged>();
    let (_module, _clock) = start(&bus);
    let wave = calibrator_at(200e-6);
    let mut codes = wave.codes(0.1e-3);
    codes[10] = 255;
    let channels = [Some(channel(codes)), None, None, None];

    bus.publish(FrameCaptured {
        frame: Arc::new(frame_of(channels, wave.interval_s)),
        context: context(200e-6),
        derived: None,
    });

    let channel = first_channel(&bus, &changes);
    assert!(channel.clipped);
    for id in [MeasurementId::Vpp, MeasurementId::RiseTime, MeasurementId::FallTime] {
        assert_eq!(cell(&channel, id), (None, Some(InvalidReason::Clipped)), "{id:?}");
    }
    assert!(cell(&channel, MeasurementId::Frequency).0.is_some());
}

#[test]
fn given_an_unverified_range_when_measured_then_the_marker_is_set_and_values_stay() {
    let bus = Bus::new();
    let changes = bus.subscribe::<MeasurementsChanged>();
    let (_module, _clock) = start(&bus);
    let mut captured = captured(&calibrator_at(200e-6), 200e-6, 0.1e-3);
    captured.context.range_verified = [false; 4];

    bus.publish(captured);

    let channel = first_channel(&bus, &changes);
    assert_eq!(
        channel.unverified.as_deref(),
        Some("Unverified range, accuracy up to about 5 % of a division.")
    );
    assert_eq!(cell(&channel, MeasurementId::Vpp).0.as_deref(), Some("1.91 V"));
}

#[test]
fn given_a_10x_probe_when_measured_then_volts_are_ten_times() {
    let bus = Bus::new();
    let changes = bus.subscribe::<MeasurementsChanged>();
    let (_module, _clock) = start(&bus);
    let mut captured = captured(&calibrator_at(200e-6), 200e-6, 0.1e-3);
    captured.context.probe = [10.0; 4];

    bus.publish(captured);

    let channel = first_channel(&bus, &changes);
    assert_eq!(cell(&channel, MeasurementId::Vpp).0.as_deref(), Some("19.1 V"));
}

#[test]
fn given_a_disabled_channel_when_measured_then_it_is_absent_and_enabled_ones_ascend() {
    let bus = Bus::new();
    let changes = bus.subscribe::<MeasurementsChanged>();
    let (_module, _clock) = start(&bus);
    let wave = calibrator_at(200e-6);
    let channels = [
        None,
        Some(channel(wave.codes(0.0))),
        None,
        Some(channel(wave.codes(0.0))),
    ];

    bus.publish(FrameCaptured {
        frame: Arc::new(frame_of(channels, wave.interval_s)),
        context: context(200e-6),
        derived: None,
    });

    let state = changes.recv_timeout(WAIT).unwrap().0;
    let numbers: Vec<u8> = state.channels.iter().map(|c| c.channel).collect();
    assert_eq!(numbers, vec![2, 4]);
}

#[test]
fn given_no_frame_yet_when_measurements_are_requested_then_the_slice_is_empty_with_the_accuracy_note() {
    let bus = Bus::new();
    let (_module, _clock) = start(&bus);

    let state = bus.request(GetMeasurements).unwrap();

    assert_eq!((state.revision, state.channels.len()), (0, 0));
    assert!(state.accuracy_note.contains("unverified"));
}

#[test]
fn given_a_change_inside_the_rate_limit_when_the_clock_passes_the_interval_then_it_is_announced() {
    let bus = Bus::new();
    let changes = bus.subscribe::<MeasurementsChanged>();
    let (_module, clock) = start(&bus);
    bus.publish(captured(&calibrator_at(200e-6), 200e-6, 0.1e-3));
    changes.recv_timeout(WAIT).expect("first");

    let mut second = captured(&calibrator_at(200e-6), 200e-6, 0.1e-3);
    second.context.probe = [10.0; 4];
    bus.publish(second);
    assert!(changes.recv_timeout(Duration::from_millis(200)).is_none());
    clock.advance(Duration::from_millis(300));

    let held_back = changes.recv_timeout(WAIT).expect("held back change").0;
    assert_eq!(held_back.revision, 2);
}

#[test]
fn given_the_same_text_again_when_a_frame_arrives_then_no_event_follows() {
    let bus = Bus::new();
    let changes = bus.subscribe::<MeasurementsChanged>();
    let (_module, clock) = start(&bus);
    bus.publish(captured(&calibrator_at(200e-6), 200e-6, 0.1e-3));
    changes.recv_timeout(WAIT).expect("first");

    clock.advance(Duration::from_secs(1));
    bus.publish(captured(&calibrator_at(200e-6), 200e-6, 0.1e-3));

    assert!(changes.recv_timeout(Duration::from_millis(300)).is_none());
}

#[test]
fn given_the_module_is_dropped_when_a_frame_is_published_then_no_event_follows() {
    let bus = Bus::new();
    let changes = bus.subscribe::<MeasurementsChanged>();
    let (module, _clock) = start(&bus);

    drop(module);
    bus.publish(captured(&calibrator_at(200e-6), 200e-6, 0.1e-3));

    assert!(changes.recv_timeout(Duration::from_millis(200)).is_none());
}
