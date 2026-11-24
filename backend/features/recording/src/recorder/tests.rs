use std::io::{self, ErrorKind, Write};
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use sweepscope_kernel::contracts::{
    ChannelData, Frame, RollCaptured, RollCloseReason, RollContext, RollItem, TriggerOutcome,
};
use sweepscope_kernel::testing::{EventRecorder, FakeClock, TempDir};
use sweepscope_kernel::Bus;

use super::{Recorder, RecorderDeps};
use crate::files::{RecordFiles, RecordWriter};
use crate::messages::RecordingChanged;
use crate::os_files::OsRecordFiles;

const FIXED_US: f64 = 1_700_000_000_000_000.0;

fn fixed_time() -> f64 {
    FIXED_US
}

#[derive(Default)]
struct Disk {
    bytes: Vec<u8>,
    fail_with: Option<ErrorKind>,
    linked: bool,
}

#[derive(Clone)]
struct FakeDisk(Arc<Mutex<Disk>>);

impl FakeDisk {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(Disk {
            linked: true,
            ..Disk::default()
        })))
    }

    fn bytes(&self) -> Vec<u8> {
        self.0.lock().unwrap().bytes.clone()
    }

    fn fail_with(&self, kind: ErrorKind) {
        self.0.lock().unwrap().fail_with = Some(kind);
    }

    fn unlink(&self) {
        self.0.lock().unwrap().linked = false;
    }

    fn records(&self) -> Vec<Record> {
        parse(&self.bytes())
    }
}

impl RecordFiles for FakeDisk {
    fn create(&self, _path: &Path) -> io::Result<Box<dyn RecordWriter>> {
        Ok(Box::new(self.clone()))
    }
}

impl Write for FakeDisk {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut disk = self.0.lock().unwrap();
        if let Some(kind) = disk.fail_with {
            return Err(io::Error::from(kind));
        }
        disk.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl RecordWriter for FakeDisk {
    fn sync(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn still_linked(&self) -> bool {
        self.0.lock().unwrap().linked
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Record {
    kind: u8,
    payload: Vec<u8>,
}

fn parse(bytes: &[u8]) -> Vec<Record> {
    assert_eq!(&bytes[..4], b"SSRL");
    assert_eq!(&bytes[4..8], [1, 0, 0, 0]);
    let mut records = Vec::new();
    let mut at = 8;
    while at + 5 <= bytes.len() {
        let length = u32::from_le_bytes(bytes[at + 1..at + 5].try_into().unwrap()) as usize;
        records.push(Record {
            kind: bytes[at],
            payload: bytes[at + 5..at + 5 + length].to_vec(),
        });
        at += 5 + length;
    }
    records
}

fn kinds(records: &[Record]) -> Vec<u8> {
    records.iter().map(|record| record.kind).collect()
}

fn u64_at(payload: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(payload[at..at + 8].try_into().unwrap())
}

struct Rig {
    bus: Bus,
    clock: FakeClock,
    disk: FakeDisk,
    recorder: Recorder,
    changes: EventRecorder<RecordingChanged>,
}

fn rig() -> Rig {
    rig_with(&FakeDisk::new())
}

fn rig_with(disk: &FakeDisk) -> Rig {
    let bus = Bus::new();
    let clock = FakeClock::default();
    let changes = EventRecorder::new(&bus);
    let recorder = Recorder::new(RecorderDeps {
        bus: bus.clone(),
        clock: Arc::new(clock.clone()),
        files: Arc::new(disk.clone()),
        unix_time_us: fixed_time,
    });
    Rig {
        bus,
        clock,
        disk: disk.clone(),
        recorder,
        changes,
    }
}

impl Rig {
    fn publish(&self, session: u32, item: RollItem) {
        self.bus.publish(RollCaptured {
            session,
            item,
            context: context(session),
        });
    }

    fn samples(&self, session: u32, first_index: u64, codes: &[u8]) {
        self.publish(
            session,
            RollItem::Samples {
                first_index,
                frame: Arc::new(frame(codes, 1.0)),
            },
        );
    }

    fn start(&mut self) {
        self.publish(1, RollItem::Opened);
        self.recorder.pump();
        self.recorder.begin(Path::new("/tmp/roll.ssroll")).unwrap();
    }

    fn one_second_later(&mut self) {
        self.clock.advance(Duration::from_secs(1));
        self.recorder.pump();
    }
}

fn context(session: u32) -> RollContext {
    RollContext {
        session,
        invert: [false; 4],
        probe: [1.0; 4],
        seconds_per_div: 0.5,
    }
}

fn frame(codes: &[u8], volts_per_div: f64) -> Frame {
    Frame {
        channels: [
            Some(ChannelData {
                codes: codes.to_vec(),
                volts_per_div,
                zero_code: 128.0,
            }),
            None,
            None,
            None,
        ],
        counts_per_div: 25.0,
        sample_interval_s: 0.002,
        trigger: TriggerOutcome::None,
    }
}

#[test]
fn given_a_roll_session_of_100_chunks_when_recorded_then_the_file_has_header_one_segment_samples_and_an_end() {
    let mut rig = rig();
    rig.start();

    for chunk in 0..100_u64 {
        rig.samples(1, chunk * 4, &[1, 2, 3, 4]);
    }
    rig.publish(
        1,
        RollItem::Closed {
            reason: RollCloseReason::Stopped,
        },
    );
    rig.recorder.pump();

    let records = rig.disk.records();
    assert_eq!(records.len(), 102);
    assert_eq!(records[0].kind, 1);
    assert!(records[1..101].iter().all(|record| record.kind == 2));
    assert_eq!(
        records[101],
        Record {
            kind: 4,
            payload: vec![0]
        }
    );
}

#[test]
fn given_a_session_when_recorded_then_the_segment_carries_the_setup_of_the_channel() {
    let mut rig = rig();
    rig.start();

    rig.samples(1, 0, &[1, 2]);
    rig.one_second_later();

    let segment: serde_json::Value = serde_json::from_slice(&rig.disk.records()[0].payload).unwrap();
    assert_eq!(segment["started_unix_us"], FIXED_US);
    assert_eq!(segment["first_index"], 0);
    assert_eq!(segment["sample_interval_s"], 0.002);
    assert_eq!(segment["counts_per_div"], 25.0);
    let channel = &segment["channels"][0];
    assert_eq!(channel["channel"], 1);
    assert_eq!(channel["volts_per_div"], 1.0);
    assert_eq!(channel["zero_code"], 128.0);
    assert_eq!(channel["inverted"], false);
}

#[test]
fn given_an_estimated_gap_when_recorded_then_a_gap_record_has_the_same_index_and_size() {
    let mut rig = rig();
    rig.start();

    rig.samples(1, 0, &[1, 2, 3, 4]);
    rig.publish(1, RollItem::Gap { after: 3, missed: 6 });
    rig.samples(1, 10, &[5, 6]);
    rig.publish(
        1,
        RollItem::Closed {
            reason: RollCloseReason::Stopped,
        },
    );
    rig.recorder.pump();

    let records = rig.disk.records();
    assert_eq!(kinds(&records), [1, 2, 3, 2, 4]);
    assert_eq!(u64_at(&records[2].payload, 0), 3);
    assert_eq!(u64_at(&records[2].payload, 8), 6);
    assert_eq!(records[2].payload[16], 1, "an overrun gap is an estimate");
}

#[test]
fn given_samples_that_skip_indices_when_recorded_then_an_exact_gap_record_fills_the_jump() {
    let mut rig = rig();
    rig.start();

    rig.samples(1, 0, &[1, 2, 3, 4]);
    rig.samples(1, 19, &[5]);
    rig.publish(
        1,
        RollItem::Closed {
            reason: RollCloseReason::Stopped,
        },
    );
    rig.recorder.pump();

    let records = rig.disk.records();
    assert_eq!(kinds(&records), [1, 2, 3, 2, 4]);
    assert_eq!(u64_at(&records[2].payload, 0), 3);
    assert_eq!(u64_at(&records[2].payload, 8), 15);
    assert_eq!(records[2].payload[16], 0, "a jump in the indices is known exactly");
}

#[test]
fn given_a_gap_before_any_samples_when_recorded_then_it_follows_the_segment_record_and_no_second_gap_is_added() {
    let mut rig = rig();
    rig.start();

    rig.publish(1, RollItem::Gap { after: 0, missed: 7 });
    rig.samples(1, 7, &[9, 9]);
    rig.publish(
        1,
        RollItem::Closed {
            reason: RollCloseReason::Stopped,
        },
    );
    rig.recorder.pump();

    let records = rig.disk.records();
    assert_eq!(kinds(&records), [1, 3, 2, 4]);
    assert_eq!(u64_at(&records[1].payload, 8), 7);
    assert_eq!(u64_at(&records[2].payload, 0), 7);
}

#[test]
fn given_a_setup_change_when_recorded_then_a_second_segment_starts_in_the_same_file() {
    let mut rig = rig();
    rig.start();

    rig.samples(1, 0, &[1, 2]);
    rig.publish(
        1,
        RollItem::Closed {
            reason: RollCloseReason::SetupChanged,
        },
    );
    rig.publish(2, RollItem::Opened);
    rig.bus.publish(RollCaptured {
        session: 2,
        item: RollItem::Samples {
            first_index: 0,
            frame: Arc::new(frame(&[7, 8], 2.0)),
        },
        context: context(2),
    });
    rig.publish(
        2,
        RollItem::Closed {
            reason: RollCloseReason::Stopped,
        },
    );
    rig.recorder.pump();

    assert_eq!(kinds(&rig.disk.records()), [1, 2, 1, 2, 4]);
    assert!(!rig.recorder.is_active());
}

#[test]
fn given_a_closed_session_when_recording_then_an_end_record_is_written_and_the_slice_is_inactive() {
    let mut rig = rig();
    rig.start();

    rig.publish(
        1,
        RollItem::Closed {
            reason: RollCloseReason::LeftRoll,
        },
    );
    rig.recorder.pump();

    assert_eq!(
        rig.disk.records().last().unwrap(),
        &Record {
            kind: 4,
            payload: vec![1]
        }
    );
    assert!(!rig.recorder.state().active);
    assert_eq!(rig.recorder.state().failure, None);
}

#[test]
fn given_a_recording_when_it_is_stopped_then_an_end_record_with_reason_stopped_is_written() {
    let mut rig = rig();
    rig.start();
    rig.samples(1, 0, &[1]);
    rig.recorder.pump();

    rig.recorder.stop();

    assert_eq!(
        rig.disk.records().last().unwrap(),
        &Record {
            kind: 4,
            payload: vec![0]
        }
    );
    assert!(!rig.recorder.state().active);
}

#[test]
fn given_samples_within_a_second_when_pumped_then_they_reach_the_file_only_at_the_next_flush() {
    let mut rig = rig();
    rig.start();
    let header_only = rig.disk.bytes().len();

    rig.samples(1, 0, &[1, 2, 3]);
    rig.recorder.pump();
    let before_flush = rig.disk.bytes().len();
    rig.one_second_later();

    assert_eq!(before_flush, header_only);
    assert!(rig.disk.bytes().len() > header_only);
}

#[test]
fn given_a_writer_that_is_full_when_the_flush_runs_then_the_recording_stops_with_the_disk_is_full() {
    let mut rig = rig();
    rig.start();
    rig.samples(1, 0, &[1, 2, 3, 4]);
    rig.recorder.pump();
    rig.disk.fail_with(ErrorKind::StorageFull);

    rig.one_second_later();

    let state = rig.recorder.state();
    let failure = state.failure.unwrap();
    assert!(!state.active);
    assert_eq!(failure.text, "the disk is full");
    assert_eq!(failure.file_name, "roll.ssroll");
    assert_eq!(
        failure.bytes, 8,
        "only the header was flushed before the disk filled up"
    );
}

#[test]
fn given_a_failed_recording_when_more_items_arrive_then_they_are_still_consumed() {
    let mut rig = rig();
    rig.start();
    rig.samples(1, 0, &[1]);
    rig.disk.fail_with(ErrorKind::StorageFull);
    rig.one_second_later();

    rig.publish(
        1,
        RollItem::Closed {
            reason: RollCloseReason::Stopped,
        },
    );
    rig.recorder.pump();

    assert!(!rig.recorder.roll_open(), "the recorder still follows the roll session");
    assert_eq!(rig.recorder.items.lagged(), 0);
    assert!(rig.recorder.items.try_next().is_none());
}

#[test]
fn given_a_removed_file_when_the_flush_runs_then_the_recording_stops_with_the_file_was_removed() {
    let mut rig = rig();
    rig.start();
    rig.samples(1, 0, &[1]);
    rig.disk.unlink();

    rig.one_second_later();

    let state = rig.recorder.state();
    assert!(!state.active);
    assert_eq!(state.failure.unwrap().text, "the file was removed");
}

#[test]
fn given_a_failure_when_a_new_recording_starts_then_the_failure_is_cleared() {
    let mut rig = rig();
    rig.start();
    rig.disk.fail_with(ErrorKind::StorageFull);
    rig.one_second_later();
    let healthy = FakeDisk::new();
    rig.recorder.files = Arc::new(healthy);

    rig.publish(2, RollItem::Opened);
    rig.recorder.pump();
    rig.recorder.begin(Path::new("/tmp/again.ssroll")).unwrap();

    assert_eq!(rig.recorder.state().failure, None);
    assert!(rig.recorder.state().active);
}

#[test]
fn given_three_hours_of_fake_time_with_a_slow_writer_when_pumped_then_the_queue_stays_bounded_and_a_gap_is_written() {
    let mut rig = rig();
    rig.start();
    let polls_per_second = 20_u64;
    let seconds = 3 * 3600_u64;
    let mut next_index = 0_u64;

    for second in 0..seconds {
        for _ in 0..polls_per_second {
            rig.samples(1, next_index, &[1, 2, 3, 4]);
            next_index += 4;
        }
        rig.clock.advance(Duration::from_secs(1));
        // The writer only gets to run every 20 s, so a second of items is dropped from the queue each time.
        if second % 20 == 19 {
            rig.recorder.pump();
        }
    }
    rig.recorder.pump();

    let records = rig.disk.records();
    let gaps = records.iter().filter(|record| record.kind == 3).count();
    let samples = records.iter().filter(|record| record.kind == 2).count();
    assert!(rig.recorder.items.try_next().is_none());
    assert!(gaps > 0, "dropped items must show as gaps");
    assert!(samples < (seconds * polls_per_second) as usize);
    assert!(
        samples <= (seconds / 20) as usize * 256 + 256,
        "at most one full queue is written per run of the writer"
    );
}

#[test]
fn given_a_recording_when_ticks_pass_then_the_slice_changes_at_most_once_a_second() {
    let mut rig = rig();
    rig.start();
    let after_start = rig.changes.events().len();

    for _ in 0..9 {
        rig.clock.advance(Duration::from_millis(100));
        rig.recorder.pump();
    }
    let within_a_second = rig.changes.events().len();
    rig.clock.advance(Duration::from_millis(100));
    rig.recorder.pump();

    assert_eq!(within_a_second, after_start);
    assert_eq!(rig.changes.events().len(), after_start + 1);
}

#[test]
fn given_a_recording_when_a_second_passes_then_the_slice_shows_elapsed_time_and_flushed_bytes() {
    let mut rig = rig();
    rig.start();
    rig.samples(1, 0, &[1, 2, 3, 4]);

    rig.one_second_later();

    let state = rig.recorder.state();
    assert!(state.active);
    assert_eq!(state.file_name.as_deref(), Some("roll.ssroll"));
    assert_eq!(state.duration.text, "0:00:01");
    assert_eq!(state.bytes_written.value, rig.disk.bytes().len() as f64);
    assert!(state.revision >= 2);
}

#[test]
fn given_no_recording_when_items_arrive_then_the_roll_session_is_tracked_and_nothing_is_written() {
    let mut rig = rig();

    rig.publish(1, RollItem::Opened);
    rig.samples(1, 0, &[1]);
    rig.recorder.pump();
    let open = rig.recorder.roll_open();
    rig.publish(
        1,
        RollItem::Closed {
            reason: RollCloseReason::Stopped,
        },
    );
    rig.recorder.pump();

    assert!(open);
    assert!(!rig.recorder.roll_open());
    assert!(rig.disk.bytes().is_empty());
}

fn python_reads(path: &Path) -> Option<serde_json::Value> {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../tools/roll-record-reader.py");
    let output = match Command::new("python3").arg("-I").arg(script).arg(path).output() {
        Ok(output) => output,
        Err(error) => {
            eprintln!("skipped: python3 is not available ({error})");
            return None;
        }
    };
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    Some(serde_json::from_slice(&output.stdout).unwrap())
}

/// Records 100 chunks of four codes with a three-sample gap after the 50th; returns the codes written.
fn record_chunks_with_a_gap(rig: &mut Rig, path: &Path) -> Vec<u8> {
    rig.recorder.files = Arc::new(OsRecordFiles);
    rig.publish(1, RollItem::Opened);
    rig.recorder.pump();
    rig.recorder.begin(path).unwrap();
    let mut written = Vec::new();
    let mut skipped = 0_u64;
    for chunk in 0..100_u32 {
        let codes: Vec<u8> = (0..4).map(|i| ((chunk * 4 + i) % 256) as u8).collect();
        rig.samples(1, u64::from(chunk) * 4 + skipped, &codes);
        written.extend(codes);
        if chunk == 49 {
            rig.publish(1, RollItem::Gap { after: 199, missed: 3 });
            skipped = 3;
        }
    }
    rig.publish(
        1,
        RollItem::Closed {
            reason: RollCloseReason::Stopped,
        },
    );
    rig.recorder.pump();
    written
}

#[test]
fn given_a_recorded_file_when_the_reference_reader_reads_it_then_every_code_and_gap_is_reproduced() {
    let dir = TempDir::new();
    let path = dir.path().join("roll.ssroll");
    let mut rig = rig();
    let expected = record_chunks_with_a_gap(&mut rig, &path);

    let Some(read) = python_reads(&path) else {
        return;
    };

    let records = read["records"].as_array().unwrap();
    let codes: Vec<u64> = records
        .iter()
        .filter(|record| record["kind"] == "samples")
        .flat_map(|record| {
            record["codes"][0]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| c.as_u64().unwrap())
        })
        .collect();
    assert_eq!(codes, expected.iter().map(|c| u64::from(*c)).collect::<Vec<_>>());
    assert_eq!(read["truncated"], false);
    assert_eq!(records.last().unwrap()["reason"], "stopped");
    let gap = records.iter().find(|record| record["kind"] == "gap").unwrap();
    assert_eq!((gap["after"].as_u64(), gap["missed"].as_u64()), (Some(199), Some(3)));
}

#[test]
fn given_a_file_cut_in_the_last_record_when_the_reference_reader_reads_it_then_it_reports_truncation() {
    let dir = TempDir::new();
    let path = dir.path().join("cut.ssroll");
    let disk = FakeDisk::new();
    let mut rig = rig_with(&disk);
    rig.start();
    rig.samples(1, 0, &[1, 2, 3, 4]);
    rig.samples(1, 4, &[5, 6, 7, 8]);
    rig.one_second_later();
    let bytes = disk.bytes();
    std::fs::write(&path, &bytes[..bytes.len() - 3]).unwrap();

    let Some(read) = python_reads(&path) else {
        return;
    };

    assert_eq!(read["truncated"], true);
    let samples = read["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["kind"] == "samples")
        .count();
    assert_eq!(samples, 1);
}
