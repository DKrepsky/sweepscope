use std::sync::Arc;
use std::time::Duration;

use sweepscope_kernel::contracts::{
    ChannelData, DerivedTrace, DerivedUnit, Frame, FrameCaptured, FrameContext, RollCaptured, RollContext, RollItem,
    TriggerOutcome,
};
use sweepscope_kernel::testing::{eventually, FakeClock};
use sweepscope_kernel::{Bus, Subscription};
use sweepscope_spectrum::{
    GetSpectrum, SetSpectrum, SpectrumDeps, SpectrumFrameReady, SpectrumModule, SpectrumPatch, SpectrumScale,
};

const WAIT: Duration = Duration::from_secs(2);
const RECORD: usize = 4096;
const COUNTS_PER_DIV: f64 = 100.0;
const ZERO_CODE: f64 = 30.0;
const SECONDS_PER_DIV: f64 = 200e-6;
const HEADER_LEN: usize = 56;

fn interval_s() -> f64 {
    SECONDS_PER_DIV * 16.4 / RECORD as f64
}

fn code_of(volts: f64) -> u8 {
    (ZERO_CODE + volts * COUNTS_PER_DIV).round() as u8
}

/// 0 to 1.91 V at 1 kHz, as the calibrator.
fn square_codes() -> Vec<u8> {
    (0..RECORD)
        .map(|i| {
            let phase = (i as f64 * interval_s()) % 1e-3;
            code_of(if phase < 0.5e-3 { 1.91 } else { 0.0 })
        })
        .collect()
}

fn constant_codes(volts: f64) -> Vec<u8> {
    vec![code_of(volts); RECORD]
}

fn channel(codes: Vec<u8>) -> ChannelData {
    ChannelData {
        codes,
        volts_per_div: 1.0,
        zero_code: ZERO_CODE,
    }
}

fn captured(ch1: Vec<u8>, derived: Option<DerivedTrace>) -> FrameCaptured {
    FrameCaptured {
        frame: Arc::new(Frame {
            channels: [Some(channel(ch1)), None, None, None],
            counts_per_div: COUNTS_PER_DIV,
            sample_interval_s: interval_s(),
            trigger: TriggerOutcome::None,
        }),
        context: FrameContext {
            setup_revision: 7,
            window_start: 0,
            window_len: RECORD,
            invert: [false; 4],
            probe: [1.0; 4],
            range_verified: [true; 4],
            seconds_per_div: SECONDS_PER_DIV,
        },
        derived: derived.map(Arc::new),
    }
}

fn math(unit: DerivedUnit) -> DerivedTrace {
    DerivedTrace {
        volts: vec![0.5; RECORD],
        unit,
        volts_per_div: 1.0,
        position_divs: 0.0,
    }
}

struct Rig {
    bus: Bus,
    frames: Subscription<SpectrumFrameReady>,
    module: SpectrumModule,
}

fn rig() -> Rig {
    let bus = Bus::new();
    let frames = bus.subscribe::<SpectrumFrameReady>();
    let clock = Arc::new(FakeClock::default());
    let module = SpectrumModule::new(SpectrumDeps {
        bus: bus.clone(),
        clock: clock.clone(),
        unix_time_us: || 1_700_000_000_000_000.0,
    })
    .unwrap();
    Rig { bus, frames, module }
}

fn activate(bus: &Bus, patch: SpectrumPatch) {
    bus.request(SetSpectrum {
        patch: SpectrumPatch {
            active: Some(true),
            ..patch
        },
    })
    .unwrap();
}

fn next_frame(rig: &Rig) -> Vec<u8> {
    rig.frames.recv_timeout(WAIT).expect("a spectrum frame").bytes.to_vec()
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn f32_at(bytes: &[u8], at: usize) -> f32 {
    f32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn f64_at(bytes: &[u8], at: usize) -> f64 {
    f64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}

fn mask(bytes: &[u8]) -> u8 {
    bytes[24]
}

fn bins(bytes: &[u8]) -> usize {
    u32_at(bytes, 28) as usize
}

fn top(bytes: &[u8]) -> f32 {
    f32_at(bytes, 40)
}

fn peak_of_first_channel(bytes: &[u8]) -> u32 {
    u32_at(bytes, HEADER_LEN)
}

fn shows_record(bus: &Bus) -> bool {
    bus.request(GetSpectrum).unwrap().record_points.value == RECORD as f64
}

#[test]
fn given_active_is_false_when_a_frame_arrives_then_no_spectrum_frame_is_produced() {
    let rig = rig();

    rig.bus.publish(captured(square_codes(), None));

    assert!(eventually(WAIT, || shows_record(&rig.bus)), "frame was not processed");
    assert_eq!(rig.frames.try_next(), None);
}

#[test]
fn given_active_and_a_1_khz_square_on_ch1_when_a_frame_arrives_then_the_frame_has_ch1_and_the_peak_at_the_fundamental()
{
    let rig = rig();
    activate(&rig.bus, SpectrumPatch::default());

    rig.bus.publish(captured(square_codes(), None));

    let bytes = next_frame(&rig);
    assert_eq!(&bytes[..4], b"SSS1");
    assert_eq!(mask(&bytes), 0b1);
    assert_eq!(bins(&bytes), 2049);
    let bin_hz = f64_at(&bytes, 32);
    let expected = 1.0 / interval_s() / RECORD as f64;
    assert!((bin_hz - expected).abs() < 1e-9, "{bin_hz} {expected}");
    assert_eq!(u32_at(&bytes, 48), 7);
    assert_eq!(peak_of_first_channel(&bytes), (1000.0 / bin_hz).round() as u32);
}

#[test]
fn given_active_and_a_math_trace_when_a_frame_arrives_then_the_math_bit_is_present() {
    let rig = rig();
    activate(&rig.bus, SpectrumPatch::default());

    rig.bus
        .publish(captured(square_codes(), Some(math(DerivedUnit::Volts))));

    assert_eq!(mask(&next_frame(&rig)), 0b1_0001);
}

#[test]
fn given_a_math_trace_in_volts_squared_when_a_frame_arrives_then_it_has_no_spectrum() {
    let rig = rig();
    activate(&rig.bus, SpectrumPatch::default());

    rig.bus
        .publish(captured(square_codes(), Some(math(DerivedUnit::VoltsSquared))));

    assert_eq!(mask(&next_frame(&rig)), 0b1);
}

#[test]
fn given_linear_and_a_peak_above_the_top_when_the_next_frame_arrives_then_the_top_steps_up_once() {
    let rig = rig();
    activate(
        &rig.bus,
        SpectrumPatch {
            scale: Some(SpectrumScale::Linear),
            ..SpectrumPatch::default()
        },
    );
    rig.bus.publish(captured(constant_codes(0.7), None));
    let first = next_frame(&rig);

    rig.bus.publish(captured(constant_codes(1.2), None));
    let second = next_frame(&rig);

    assert_eq!(top(&first), 1.0);
    assert_eq!(top(&second), 2.0);
    assert_eq!(bytes_flag_linear(&second), 1);
}

fn bytes_flag_linear(bytes: &[u8]) -> u8 {
    bytes[25] & 1
}

#[test]
fn given_dbv_when_a_frame_arrives_then_the_top_is_plus_10_and_the_ticks_are_every_division() {
    let rig = rig();
    activate(&rig.bus, SpectrumPatch::default());

    rig.bus.publish(captured(square_codes(), None));

    let bytes = next_frame(&rig);
    assert_eq!(top(&bytes), 10.0);
    assert_eq!(f32_at(&bytes, 44), 10.0);
    let state = rig.bus.request(GetSpectrum).unwrap();
    assert_eq!(state.top_label, "+10 dBV");
    assert_eq!(state.ticks_y.len(), 9);
    assert_eq!(state.ticks_y[1].at, 0.125);
}

#[test]
fn given_a_frame_when_the_slice_is_read_then_the_readouts_follow_the_record() {
    let rig = rig();
    rig.bus.publish(captured(square_codes(), None));

    assert!(eventually(WAIT, || shows_record(&rig.bus)));

    let state = rig.bus.request(GetSpectrum).unwrap();
    let sample_rate = 1.0 / interval_s();
    assert!((state.sample_rate.value - sample_rate).abs() < 1e-6);
    assert!((state.span.value - sample_rate / 2.0).abs() < 1e-6);
    assert!((state.bin_spacing.value - sample_rate / RECORD as f64).abs() < 1e-9);
    assert_eq!(state.ticks_x.len(), 6);
    assert_eq!(state.ticks_x[5].at, 1.0);
}

#[test]
fn given_the_window_changed_when_the_capture_is_held_then_the_last_frame_is_computed_again() {
    let rig = rig();
    activate(&rig.bus, SpectrumPatch::default());
    rig.bus.publish(captured(square_codes(), None));
    let hann = next_frame(&rig);

    rig.bus
        .request(SetSpectrum {
            patch: SpectrumPatch {
                window: Some(sweepscope_spectrum::SpectrumWindow::Blackman),
                ..SpectrumPatch::default()
            },
        })
        .unwrap();

    let blackman = next_frame(&rig);
    assert_eq!(hann[26], 1);
    assert_eq!(blackman[26], 3);
}

#[test]
fn given_a_roll_session_when_it_opens_then_the_view_is_unavailable_with_the_reason() {
    let rig = rig();
    activate(&rig.bus, SpectrumPatch::default());
    rig.bus.publish(captured(square_codes(), None));
    next_frame(&rig);

    rig.bus.publish(RollCaptured {
        session: 1,
        item: RollItem::Opened,
        context: RollContext {
            session: 1,
            invert: [false; 4],
            probe: [1.0; 4],
            seconds_per_div: 0.5,
        },
    });

    assert!(eventually(WAIT, || {
        !rig.bus.request(GetSpectrum).unwrap().availability.enabled
    }));
    let reason = rig.bus.request(GetSpectrum).unwrap().availability.reason;
    assert_eq!(
        reason.as_deref(),
        Some("FFT is not available in roll mode: it needs a full record.")
    );
}

#[test]
fn given_a_running_module_when_it_is_dropped_then_its_thread_ends() {
    let rig = rig();

    drop(rig.module);

    assert!(rig.bus.request(GetSpectrum).is_ok());
}
