use std::sync::Arc;
use std::time::Duration;

use sweepscope_kernel::contracts::{
    ChannelData, Frame, RollCaptured, RollCloseReason, RollContext, RollItem, TriggerOutcome,
};
use sweepscope_kernel::testing::{eventually, FakeClock, ScriptedPicker, TempDir};
use sweepscope_kernel::{Bus, ErrorCategory};
use sweepscope_recording::{
    GetRecording, OsRecordFiles, RecordingDeps, RecordingModule, StartRecording, StopRecording,
};

const NOW_US: f64 = 1_700_000_000_000_000.0;
const WAIT: Duration = Duration::from_secs(5);

fn clock() -> f64 {
    NOW_US
}

struct Rig {
    bus: Bus,
    clock: FakeClock,
    dir: TempDir,
    picker: Arc<ScriptedPicker>,
    _module: RecordingModule,
}

fn rig(names: &[Option<&str>]) -> Rig {
    let dir = TempDir::new();
    let answers = names
        .iter()
        .map(|name| name.map(|name| dir.path().join(name)))
        .collect();
    let bus = Bus::new();
    let fake_clock = FakeClock::default();
    let picker = Arc::new(ScriptedPicker::new(answers, vec![]));
    let module = RecordingModule::new(RecordingDeps {
        bus: bus.clone(),
        clock: Arc::new(fake_clock.clone()),
        picker: picker.clone(),
        files: Arc::new(OsRecordFiles),
        unix_time_us: clock,
    })
    .unwrap();
    Rig {
        bus,
        clock: fake_clock,
        dir,
        picker,
        _module: module,
    }
}

fn publish(bus: &Bus, item: RollItem) {
    bus.publish(RollCaptured {
        session: 1,
        item,
        context: RollContext {
            session: 1,
            invert: [false; 4],
            probe: [1.0; 4],
            seconds_per_div: 0.5,
        },
    });
}

fn samples(first_index: u64) -> RollItem {
    RollItem::Samples {
        first_index,
        frame: Arc::new(Frame {
            channels: [
                Some(ChannelData {
                    codes: vec![10, 20, 30],
                    volts_per_div: 1.0,
                    zero_code: 128.0,
                }),
                None,
                None,
                None,
            ],
            counts_per_div: 25.0,
            sample_interval_s: 0.002,
            trigger: TriggerOutcome::None,
        }),
    }
}

#[test]
fn given_no_roll_session_when_recording_is_started_then_it_is_rejected_and_no_dialog_opens() {
    let rig = rig(&[]);

    let error = rig.bus.request(StartRecording).unwrap_err();

    assert_eq!(error.category, ErrorCategory::Recoverable);
    assert_eq!(error.message, "Recording needs roll mode (500 ms/div or slower)");
    assert!(rig.picker.suggested_names().is_empty());
}

#[test]
fn given_a_picker_that_cancels_when_recording_is_started_then_the_answer_is_none_and_no_file_exists() {
    let rig = rig(&[None]);
    publish(&rig.bus, RollItem::Opened);

    let saved = rig.bus.request(StartRecording).unwrap();

    assert_eq!(saved, None);
    assert_eq!(std::fs::read_dir(rig.dir.path()).unwrap().count(), 0);
    assert!(!rig.bus.request(GetRecording).unwrap().active);
}

#[test]
fn given_a_roll_session_when_recording_is_started_then_the_dialog_suggests_the_dated_name_and_the_slice_is_active() {
    let rig = rig(&[Some("take.ssroll")]);
    publish(&rig.bus, RollItem::Opened);

    let saved = rig.bus.request(StartRecording).unwrap().unwrap();

    assert_eq!(saved.file_name, "take.ssroll");
    assert_eq!(rig.picker.suggested_names(), ["roll-2023-11-14-221320.ssroll"]);
    assert!(rig.bus.request(GetRecording).unwrap().active);
}

#[test]
fn given_a_recording_when_it_is_started_again_then_it_is_rejected() {
    let rig = rig(&[Some("take.ssroll")]);
    publish(&rig.bus, RollItem::Opened);
    rig.bus.request(StartRecording).unwrap();

    let error = rig.bus.request(StartRecording).unwrap_err();

    assert_eq!(error.message, "Recording is already running");
}

#[test]
fn given_a_recording_when_chunks_arrive_and_a_second_passes_then_the_worker_flushes_them_to_the_file() {
    let rig = rig(&[Some("take.ssroll")]);
    publish(&rig.bus, RollItem::Opened);
    rig.bus.request(StartRecording).unwrap();

    publish(&rig.bus, samples(0));
    rig.clock.advance(Duration::from_secs(1));

    let path = rig.dir.path().join("take.ssroll");
    assert!(eventually(WAIT, || std::fs::metadata(&path).is_ok_and(|m| m.len() > 8)));
    assert!(eventually(WAIT, || rig
        .bus
        .request(GetRecording)
        .unwrap()
        .bytes_written
        .value
        > 8.0));
}

#[test]
fn given_a_recording_when_the_file_is_deleted_then_the_next_flush_stops_it_with_the_file_was_removed() {
    let rig = rig(&[Some("take.ssroll")]);
    publish(&rig.bus, RollItem::Opened);
    rig.bus.request(StartRecording).unwrap();
    std::fs::remove_file(rig.dir.path().join("take.ssroll")).unwrap();

    rig.clock.advance(Duration::from_secs(1));

    assert!(eventually(WAIT, || !rig.bus.request(GetRecording).unwrap().active));
    let failure = rig.bus.request(GetRecording).unwrap().failure.unwrap();
    assert_eq!(failure.text, "the file was removed");
    assert_eq!(failure.file_name, "take.ssroll");
}

#[test]
fn given_a_recording_when_the_session_closes_then_the_slice_becomes_inactive_without_a_failure() {
    let rig = rig(&[Some("take.ssroll")]);
    publish(&rig.bus, RollItem::Opened);
    rig.bus.request(StartRecording).unwrap();

    publish(
        &rig.bus,
        RollItem::Closed {
            reason: RollCloseReason::Disconnected,
        },
    );

    assert!(eventually(WAIT, || !rig.bus.request(GetRecording).unwrap().active));
    assert_eq!(rig.bus.request(GetRecording).unwrap().failure, None);
}

#[test]
fn given_a_recording_when_it_is_stopped_then_the_file_ends_with_an_end_record() {
    let rig = rig(&[Some("take.ssroll")]);
    publish(&rig.bus, RollItem::Opened);
    rig.bus.request(StartRecording).unwrap();
    publish(&rig.bus, samples(0));

    rig.bus.request(StopRecording).unwrap();

    let bytes = std::fs::read(rig.dir.path().join("take.ssroll")).unwrap();
    assert_eq!(&bytes[..4], b"SSRL");
    assert_eq!(bytes[bytes.len() - 6..], [4, 1, 0, 0, 0, 0]);
    assert!(!rig.bus.request(GetRecording).unwrap().active);
}
