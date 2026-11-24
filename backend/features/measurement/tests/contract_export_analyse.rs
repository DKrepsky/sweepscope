use std::path::Path;

use sweepscope_measurement::{MeasurementsState, Warning, WarningSeverity};
use ts_rs::{Config, TS};

fn generated(name: &str) -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../frontend/src/api/generated")
        .join(name);
    std::fs::read_to_string(file).unwrap()
}

#[test]
fn given_the_measurements_slice_when_exported_then_it_lists_warnings_with_a_badge_and_a_tooltip_text() {
    let config = Config::from_env();
    MeasurementsState::export_all(&config).unwrap();
    Warning::export_all(&config).unwrap();
    WarningSeverity::export_all(&config).unwrap();

    assert!(generated("MeasurementsState.ts").contains("warnings: Array<Warning>"));
    let warning = generated("Warning.ts");
    for field in [
        "id: string",
        "channel: number | null",
        "short: string",
        "message: string",
    ] {
        assert!(warning.contains(field), "{field} missing in {warning}");
    }
    assert!(generated("WarningSeverity.ts").contains("\"warning\""));
}
