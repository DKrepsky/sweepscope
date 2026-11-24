use std::path::Path;

use sweepscope_recording::{RecordingFailure, RecordingState};
use ts_rs::{Config, TS};

fn generated(name: &str) -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../frontend/src/api/generated")
        .join(name);
    std::fs::read_to_string(file).unwrap()
}

#[test]
fn given_the_recording_slice_when_exported_then_it_has_the_chip_data_and_the_failure() {
    let config = Config::from_env();
    RecordingState::export_all(&config).unwrap();
    RecordingFailure::export_all(&config).unwrap();

    let state = generated("RecordingState.ts");
    for field in [
        "active: boolean",
        "fileName: string | null",
        "bytesWritten: FormattedValue",
        "failure: RecordingFailure | null",
    ] {
        assert!(state.contains(field), "{field} missing in {state}");
    }
    assert!(generated("RecordingFailure.ts").contains("bytes: number"));
}
