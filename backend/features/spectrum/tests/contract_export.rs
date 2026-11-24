use std::path::Path;

use sweepscope_spectrum::{SpectrumPatch, SpectrumScale, SpectrumState, SpectrumWindow};
use ts_rs::{Config, TS};

fn generated(name: &str) -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../frontend/src/api/generated")
        .join(name);
    std::fs::read_to_string(file).unwrap()
}

#[test]
fn given_the_spectrum_slice_when_exported_then_it_has_the_axis_texts_and_the_availability() {
    let config = Config::from_env();
    SpectrumState::export_all(&config).unwrap();
    SpectrumScale::export_all(&config).unwrap();
    SpectrumWindow::export_all(&config).unwrap();

    let state = generated("SpectrumState.ts");
    for field in [
        "active: boolean",
        "ticksX: Array<Tick>",
        "topLabel: string",
        "availability: Availability",
    ] {
        assert!(state.contains(field), "{field} missing in {state}");
    }
    assert!(generated("SpectrumScale.ts").contains("\"dbv\" | \"linear\""));
    assert!(generated("SpectrumWindow.ts").contains("\"blackman\""));
}

#[test]
fn given_a_json_patch_with_one_field_when_deserialized_then_the_others_are_none() {
    let patch: SpectrumPatch = serde_json::from_str(r#"{"active": true}"#).unwrap();

    assert_eq!(
        patch,
        SpectrumPatch {
            active: Some(true),
            ..SpectrumPatch::default()
        }
    );
}
