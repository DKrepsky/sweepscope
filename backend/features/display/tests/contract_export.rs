use std::path::Path;

use sweepscope_display::{DisplayState, InterpolationMode};
use ts_rs::{Config, TS};

fn generated(name: &str) -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../frontend/src/api/generated")
        .join(name);
    std::fs::read_to_string(file).unwrap()
}

#[test]
fn given_the_display_types_when_exported_then_modes_are_lower_camel_strings_and_the_revision_is_a_number() {
    let config = Config::from_env();
    InterpolationMode::export_all(&config).unwrap();
    DisplayState::export_all(&config).unwrap();

    assert!(generated("InterpolationMode.ts").contains("\"sinc\" | \"linear\" | \"dots\""));
    let state = generated("DisplayState.ts");
    assert!(state.contains("revision: number"), "{state}");
    assert!(state.contains("interpolation: InterpolationMode"), "{state}");
    assert!(state.contains("automatic: boolean"), "{state}");
}
