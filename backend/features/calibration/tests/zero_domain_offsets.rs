use sweepscope_calibration::domain::offsets::{OffsetFile, StoredUnit, CORRUPT_KEY, FILE_KEY};
use sweepscope_kernel::testing::MemoryStorage;
use sweepscope_kernel::{KernelError, Storage};

fn unit(seed: f64) -> StoredUnit {
    StoredUnit {
        stored_at_utc: "2026-11-24T10:00:00Z".to_owned(),
        volts: [vec![seed], vec![seed * 2.0], vec![], vec![seed * 4.0, 0.0]],
    }
}

struct FailingWrites;

impl Storage for FailingWrites {
    fn read(&self, _key: &str) -> Result<Option<Vec<u8>>, KernelError> {
        Ok(None)
    }

    fn write(&self, _key: &str, _bytes: &[u8]) -> Result<(), KernelError> {
        Err(KernelError::recoverable("disk full"))
    }
}

#[test]
fn given_no_file_when_loaded_then_no_unit_has_offsets() {
    let file = OffsetFile::load(&MemoryStorage::default()).unwrap();

    assert_eq!(file.offsets_for("family/pcb/1"), None);
}

#[test]
fn given_a_saved_unit_when_loaded_again_then_the_offsets_round_trip() {
    let storage = MemoryStorage::default();
    OffsetFile::default()
        .with_unit("family/pcb/1", unit(0.001))
        .save(&storage)
        .unwrap();

    let offsets = OffsetFile::load(&storage).unwrap().offsets_for("family/pcb/1").unwrap();

    assert_eq!(offsets.unit_id, "family/pcb/1");
    assert_eq!(offsets.volts, unit(0.001).volts);
}

#[test]
fn given_a_file_with_two_units_when_one_is_connected_then_only_its_offsets_apply() {
    let storage = MemoryStorage::default();
    OffsetFile::default()
        .with_unit("A", unit(0.001))
        .with_unit("B", unit(0.5))
        .save(&storage)
        .unwrap();
    let file = OffsetFile::load(&storage).unwrap();

    assert_eq!(file.offsets_for("B").unwrap().volts[0], vec![0.5]);
    assert_eq!(file.offsets_for("A").unwrap().volts[0], vec![0.001]);
    assert_eq!(file.offsets_for("C"), None);
}

#[test]
fn given_a_stored_unit_when_another_is_committed_then_the_first_stays() {
    let storage = MemoryStorage::default();
    OffsetFile::default()
        .with_unit("A", unit(0.001))
        .save(&storage)
        .unwrap();

    OffsetFile::load(&storage)
        .unwrap()
        .with_unit("B", unit(0.5))
        .save(&storage)
        .unwrap();

    let file = OffsetFile::load(&storage).unwrap();
    assert!(file.unit("A").is_some() && file.unit("B").is_some());
}

#[test]
fn given_a_corrupt_file_when_loaded_then_it_is_copied_once_and_treated_as_empty() {
    let storage = MemoryStorage::default();
    storage.write(FILE_KEY, b"{ not json").unwrap();

    let file = OffsetFile::load(&storage).unwrap();
    storage.write(FILE_KEY, b"second garbage").unwrap();
    OffsetFile::load(&storage).unwrap();

    assert_eq!(file.offsets_for("A"), None);
    assert_eq!(storage.read(CORRUPT_KEY).unwrap(), Some(b"{ not json".to_vec()));
}

#[test]
fn given_a_file_of_another_version_when_loaded_then_it_is_treated_as_corrupt() {
    let storage = MemoryStorage::default();
    let foreign = br#"{"format":"sweepscope-zero-offsets","version":2,"units":{}}"#;
    storage.write(FILE_KEY, foreign).unwrap();

    let file = OffsetFile::load(&storage).unwrap();

    assert_eq!(file.offsets_for("A"), None);
    assert_eq!(storage.read(CORRUPT_KEY).unwrap(), Some(foreign.to_vec()));
}

#[test]
fn given_a_failing_storage_when_saved_then_the_error_is_returned() {
    let result = OffsetFile::default().with_unit("A", unit(0.001)).save(&FailingWrites);

    assert_eq!(result, Err(KernelError::recoverable("disk full")));
}

fn stored_with(value: &str) -> Vec<u8> {
    format!(
        r#"{{"format":"sweepscope-zero-offsets","version":1,"units":{{"A":{{"stored_at_utc":"t","volts":[[{value}],[],[],[]]}}}}}}"#
    )
    .into_bytes()
}

#[test]
fn given_an_offset_beyond_any_range_when_loaded_then_it_is_treated_as_corrupt() {
    let storage = MemoryStorage::default();
    let bytes = stored_with("1e9");
    storage.write(FILE_KEY, &bytes).unwrap();

    let file = OffsetFile::load(&storage).unwrap();

    assert_eq!(file.offsets_for("A"), None);
    assert_eq!(storage.read(CORRUPT_KEY).unwrap(), Some(bytes));
}

#[test]
fn given_a_non_finite_offset_when_loaded_then_it_is_treated_as_corrupt() {
    let storage = MemoryStorage::default();
    storage.write(FILE_KEY, &stored_with("1e999")).unwrap();

    let file = OffsetFile::load(&storage).unwrap();

    assert_eq!(file.offsets_for("A"), None);
    assert!(storage.read(CORRUPT_KEY).unwrap().is_some());
}

#[test]
fn given_an_offset_at_the_bound_when_loaded_then_it_is_kept() {
    let storage = MemoryStorage::default();
    storage.write(FILE_KEY, &stored_with("-40.0")).unwrap();

    let file = OffsetFile::load(&storage).unwrap();

    assert!(file.offsets_for("A").is_some());
}
