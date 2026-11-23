use std::path::Path;

use sweepscope_measurement::{
    ChannelMeasurements, Invalid, InvalidReason, Measurement, MeasurementId, MeasurementsState,
};
use ts_rs::{Config, TS};

fn generated(name: &str) -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../frontend/src/api/generated")
        .join(name);
    std::fs::read_to_string(file).unwrap()
}

#[test]
fn given_the_measurement_slice_when_exported_then_the_revision_is_a_number_and_fields_are_camel_case() {
    let config = Config::from_env();
    MeasurementsState::export_all(&config).unwrap();
    ChannelMeasurements::export_all(&config).unwrap();
    Measurement::export_all(&config).unwrap();
    Invalid::export_all(&config).unwrap();

    let state = generated("MeasurementsState.ts");
    assert!(state.contains("revision: number"), "{state}");
    assert!(state.contains("accuracyNote: string"), "{state}");
    assert!(generated("ChannelMeasurements.ts").contains("unverified: string | null"));
    let measurement = generated("Measurement.ts");
    assert!(measurement.contains("value: FormattedValue | null"), "{measurement}");
    assert!(measurement.contains("invalid: Invalid | null"), "{measurement}");
}

#[test]
fn given_the_id_and_reason_enums_when_exported_then_they_are_lower_camel_strings() {
    let config = Config::from_env();
    MeasurementId::export_all(&config).unwrap();
    InvalidReason::export_all(&config).unwrap();

    let ids = generated("MeasurementId.ts");
    assert!(
        ids.contains("\"positiveWidth\"") && ids.contains("\"dutyCycle\""),
        "{ids}"
    );
    assert!(generated("InvalidReason.ts").contains("\"fewerThanOnePeriod\""));
}
