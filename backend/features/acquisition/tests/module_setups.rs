#[path = "domain_support/mod.rs"]
mod domain_support;
mod module_support;

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use module_support::{expect_recoverable, rig, rig_with, Rig};
use serde_json::Value;
use sweepscope_acquisition::{InstrumentState, LoadSetup, SaveSetup, SetRunning, SettingTarget, StepSetting};
use sweepscope_kernel::testing::{MemoryStorage, ScriptedPicker, TempDir};
use sweepscope_kernel::{ErrorCategory, Storage};

const CH1: SettingTarget = SettingTarget::Range { channel: 1 };
const LAST_SETUP: &str = "last-setup.json";
const CORRUPT: &str = "last-setup.json.corrupt";

struct Files {
    dir: TempDir,
}

impl Files {
    fn new() -> Self {
        Self { dir: TempDir::new() }
    }

    fn path(&self) -> PathBuf {
        self.dir.path().join("my-setup.json")
    }

    fn picker(&self, saves: usize, loads: usize) -> ScriptedPicker {
        ScriptedPicker::new(vec![Some(self.path()); saves], vec![Some(self.path()); loads])
    }
}

fn instrument_of(rig: &Rig) -> InstrumentState {
    rig.snapshot().instrument.unwrap()
}

fn without_revision(mut state: InstrumentState) -> InstrumentState {
    state.revision = 0;
    state
}

fn step_range(rig: &Rig, direction: i8) {
    rig.bus.request(StepSetting { target: CH1, direction }).unwrap();
}

fn written(files: &Files) -> Value {
    serde_json::from_slice(&fs::read(files.path()).unwrap()).unwrap()
}

fn stored(storage: &MemoryStorage, key: &str) -> Option<Value> {
    storage
        .read(key)
        .unwrap()
        .map(|bytes| serde_json::from_slice(&bytes).unwrap())
}

#[test]
fn given_a_ready_simulator_when_a_setup_is_saved_then_the_file_is_pretty_json_in_volts_and_seconds() {
    let files = Files::new();
    let rig = rig_with(MemoryStorage::default(), files.picker(1, 0));
    rig.plug();

    let saved = rig.bus.request(SaveSetup).unwrap().unwrap();

    assert_eq!(saved.file_name, "my-setup.json");
    let text = fs::read_to_string(files.path()).unwrap();
    assert!(text.contains("\n  \"format\": \"sweepscope-setup\""), "{text}");
    let file = written(&files);
    assert_eq!(
        (file["version"].clone(), file["channels"][0]["volts_per_div"].clone()),
        (1.into(), 1.0.into())
    );
    assert_eq!(file["time_base_s_per_div"], 200e-6);
}

#[test]
fn given_a_save_when_the_dialog_is_cancelled_then_nothing_is_written() {
    let files = Files::new();
    let rig = rig_with(MemoryStorage::default(), ScriptedPicker::new(vec![None], vec![]));
    rig.plug();

    let saved = rig.bus.request(SaveSetup).unwrap();

    assert!(saved.is_none());
    assert!(!files.path().exists());
}

#[test]
fn given_a_save_when_the_folder_is_missing_then_the_error_names_the_file_and_the_cause() {
    let files = Files::new();
    let missing = files.dir.path().join("gone").join("s.json");
    let rig = rig_with(
        MemoryStorage::default(),
        ScriptedPicker::new(vec![Some(missing)], vec![]),
    );
    rig.plug();

    let error = rig.bus.request(SaveSetup).unwrap_err();

    assert_eq!(error.category, ErrorCategory::Recoverable);
    assert_eq!(error.message, "cannot write s.json: the folder does not exist");
}

#[test]
fn given_no_device_ever_ready_when_a_setup_is_saved_then_it_fails_with_no_scope_connected() {
    let files = Files::new();
    let rig = rig_with(MemoryStorage::default(), files.picker(1, 0));

    let reason = expect_recoverable(rig.bus.request(SaveSetup).map(drop));

    assert_eq!(reason, "No scope connected");
}

#[test]
fn given_a_saved_setup_and_a_changed_instrument_when_loaded_then_the_instrument_equals_the_saved_one() {
    let files = Files::new();
    let rig = rig_with(MemoryStorage::default(), files.picker(1, 1));
    rig.plug();
    step_range(&rig, -1);
    let saved_state = without_revision(instrument_of(&rig));
    rig.bus.request(SaveSetup).unwrap();
    step_range(&rig, 1);
    step_range(&rig, 1);

    let loaded = rig.bus.request(LoadSetup).unwrap().unwrap();

    assert_eq!(loaded.file_name, "my-setup.json");
    assert!(loaded.warnings.is_empty());
    assert_eq!(without_revision(instrument_of(&rig)), saved_state);
}

#[test]
fn given_a_load_when_it_changes_the_setup_then_the_device_receives_it() {
    let files = Files::new();
    let rig = rig_with(MemoryStorage::default(), files.picker(1, 1));
    rig.plug();
    rig.bus.request(SaveSetup).unwrap();
    step_range(&rig, 1);
    let applies = rig.device.applies().len();

    rig.bus.request(LoadSetup).unwrap();

    assert_eq!(rig.device.applies().len(), applies + 1);
}

#[test]
fn given_a_file_with_a_range_of_3_volts_when_loaded_then_the_current_value_stays_and_a_warning_names_it() {
    let files = Files::new();
    let rig = rig_with(MemoryStorage::default(), files.picker(1, 1));
    rig.plug();
    rig.bus.request(SaveSetup).unwrap();
    let mut file = written(&files);
    file["channels"][0]["volts_per_div"] = 3.0.into();
    fs::write(files.path(), serde_json::to_vec(&file).unwrap()).unwrap();
    let before = instrument_of(&rig);

    let loaded = rig.bus.request(LoadSetup).unwrap().unwrap();

    assert_eq!(loaded.warnings, ["CH1 volts/div 3.00 V is not available; kept 1.00 V"]);
    assert_eq!(without_revision(instrument_of(&rig)), without_revision(before));
}

fn assert_load_fails_and_changes_nothing(content: &[u8], reason: &str) {
    let files = Files::new();
    let rig = rig_with(MemoryStorage::default(), files.picker(0, 1));
    rig.plug();
    fs::write(files.path(), content).unwrap();
    let before = instrument_of(&rig);
    let applies = rig.device.applies().len();

    let error = rig.bus.request(LoadSetup).unwrap_err();

    assert_eq!(error.category, ErrorCategory::Recoverable);
    assert!(error.message.contains(reason), "{}", error.message);
    assert_eq!(instrument_of(&rig), before);
    assert_eq!(rig.device.applies().len(), applies);
}

#[test]
fn given_a_file_that_is_not_json_when_loaded_then_the_error_gives_the_reason_and_the_setup_is_unchanged() {
    assert_load_fails_and_changes_nothing(b"{nope", "not valid JSON");
}

#[test]
fn given_a_file_of_another_format_when_loaded_then_the_error_gives_the_reason_and_the_setup_is_unchanged() {
    assert_load_fails_and_changes_nothing(br#"{"format":"other","version":1}"#, "not a SweepScope setup");
}

#[test]
fn given_a_file_of_version_2_when_loaded_then_the_error_gives_the_reason_and_the_setup_is_unchanged() {
    assert_load_fails_and_changes_nothing(
        br#"{"format":"sweepscope-setup","version":2}"#,
        "newer SweepScope (version 2)",
    );
}

#[test]
fn given_a_file_larger_than_the_limit_when_loaded_then_the_error_says_it_is_too_large() {
    assert_load_fails_and_changes_nothing(&vec![b' '; 2 * 1024 * 1024], "file is too large");
}

#[test]
fn given_a_load_when_the_dialog_is_cancelled_then_nothing_changes() {
    let rig = rig_with(MemoryStorage::default(), ScriptedPicker::new(vec![], vec![None]));
    rig.plug();

    let loaded = rig.bus.request(LoadSetup).unwrap();

    assert!(loaded.is_none());
}

#[test]
fn given_no_device_when_a_load_is_requested_then_it_fails_with_no_scope_connected() {
    let files = Files::new();
    let rig = rig_with(MemoryStorage::default(), files.picker(0, 1));

    let reason = expect_recoverable(rig.bus.request(LoadSetup).map(drop));

    assert_eq!(reason, "No scope connected");
}

#[test]
fn given_a_device_that_was_unplugged_when_a_load_is_requested_then_it_fails_with_no_scope_connected() {
    let files = Files::new();
    let rig = rig_with(MemoryStorage::default(), files.picker(0, 1));
    rig.plug();
    rig.unplug();

    let reason = expect_recoverable(rig.bus.request(LoadSetup).map(drop));

    assert_eq!(reason, "No scope connected");
}

#[test]
fn given_a_change_when_one_second_has_passed_then_last_setup_json_exists() {
    let rig = rig();
    rig.plug();

    step_range(&rig, -1);
    assert!(rig.storage.read(LAST_SETUP).unwrap().is_none());
    rig.clock.advance(Duration::from_secs(1));

    rig.wait(|| rig.storage.read(LAST_SETUP).unwrap().is_some());
    let file = stored(&rig.storage, LAST_SETUP).unwrap();
    assert_eq!(file["format"], "sweepscope-setup");
    assert_eq!(file["channels"][0]["volts_per_div"], 0.5);
}

#[test]
fn given_a_change_when_less_than_a_second_has_passed_then_last_setup_json_is_not_written() {
    let rig = rig();
    rig.plug();
    rig.bus.request(SetRunning { running: true }).unwrap();

    step_range(&rig, -1);
    rig.clock.advance(Duration::from_millis(900));
    rig.settle_after_capture_events();

    assert!(rig.storage.read(LAST_SETUP).unwrap().is_none());
}

#[test]
fn given_a_change_when_the_module_is_dropped_then_the_setup_is_flushed() {
    let rig = rig();
    rig.plug();
    step_range(&rig, -1);

    let Rig { module, storage, .. } = rig;
    drop(module);

    assert_eq!(
        stored(&storage, LAST_SETUP).unwrap()["channels"][0]["volts_per_div"],
        0.5
    );
}

#[test]
fn given_a_new_module_on_the_same_storage_when_the_first_device_is_ready_then_the_instrument_equals_the_last_setup_and_is_stopped(
) {
    let storage = MemoryStorage::default();
    let first = rig_with(storage.clone(), ScriptedPicker::default());
    first.plug();
    step_range(&first, -1);
    first.bus.request(SetRunning { running: true }).unwrap();
    let expected = without_revision(instrument_of(&first));
    drop(first);

    let second = rig_with(storage, ScriptedPicker::default());
    second.plug();

    let restored = instrument_of(&second);
    assert!(!restored.running);
    assert_eq!(second.device.capturing_calls(), Vec::<bool>::new());
    assert_eq!(restored.channels, expected.channels);
    assert_eq!(restored.horizontal, expected.horizontal);
    assert_eq!(restored.trigger, expected.trigger);
    assert!(instrument_of(&second).setup_notice.is_none());
    assert_eq!(
        second.device.applies().last().unwrap().1.channels[0].range_index,
        domain_support::range_index(&domain_support::capabilities(), 0.5)
    );
}

#[test]
fn given_a_corrupt_last_setup_when_the_first_device_is_ready_then_defaults_apply_the_file_is_kept_and_a_notice_is_set()
{
    let storage = MemoryStorage::default();
    storage.write(LAST_SETUP, b"{broken").unwrap();
    let rig = rig_with(storage.clone(), ScriptedPicker::default());
    rig.plug();

    let state = instrument_of(&rig);

    assert_eq!(
        state.channels[0].range.index,
        domain_support::range_index(&domain_support::capabilities(), 1.0)
    );
    assert_eq!(storage.read(CORRUPT).unwrap().unwrap(), b"{broken");
    let notice = state.setup_notice.unwrap();
    assert!(notice.text.contains("could not be restored"), "{}", notice.text);
}

#[test]
fn given_a_corrupt_backup_already_exists_when_another_corrupt_file_is_found_then_the_first_backup_is_kept() {
    let storage = MemoryStorage::default();
    storage.write(CORRUPT, b"first").unwrap();
    storage.write(LAST_SETUP, b"second").unwrap();

    let rig = rig_with(storage.clone(), ScriptedPicker::default());
    rig.plug();

    assert_eq!(storage.read(CORRUPT).unwrap().unwrap(), b"first");
}

#[test]
fn given_a_last_setup_with_an_unavailable_value_when_the_first_device_is_ready_then_the_notice_names_it() {
    let storage = MemoryStorage::default();
    let seed = rig_with(storage.clone(), ScriptedPicker::default());
    seed.plug();
    let Rig { module, .. } = seed;
    drop(module);
    let mut file = stored(&storage, LAST_SETUP).unwrap();
    file["channels"][0]["volts_per_div"] = 3.0.into();
    storage.write(LAST_SETUP, &serde_json::to_vec(&file).unwrap()).unwrap();

    let rig = rig_with(storage, ScriptedPicker::default());
    rig.plug();

    let notice = instrument_of(&rig).setup_notice.unwrap();
    assert!(
        notice.text.contains("CH1 volts/div 3.00 V is not available"),
        "{}",
        notice.text
    );
}

#[test]
fn given_a_restored_setup_when_the_device_reconnects_then_the_notice_keeps_its_id() {
    let storage = MemoryStorage::default();
    storage.write(LAST_SETUP, b"{broken").unwrap();
    let rig = rig_with(storage, ScriptedPicker::default());
    rig.plug();
    let first = instrument_of(&rig).setup_notice.unwrap();

    rig.unplug();
    rig.plug();

    assert_eq!(instrument_of(&rig).setup_notice.unwrap().id, first.id);
}

#[test]
fn given_the_instrument_slice_when_a_notice_exists_then_it_is_part_of_the_published_state() {
    let storage = MemoryStorage::default();
    storage.write(LAST_SETUP, b"{broken").unwrap();
    let rig = rig_with(storage, ScriptedPicker::default());

    rig.plug();

    let events = rig.instrument.events();
    assert!(events.last().unwrap().0.setup_notice.is_some());
}
