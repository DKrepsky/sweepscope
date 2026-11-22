use std::path::Path;

use sweepscope_device::{CalibrationStatus, Capabilities, DeviceIdentity, TimeBaseStep, VerticalRange};
use ts_rs::{Config, TS};

fn generated(name: &str) -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../frontend/src/api/generated")
        .join(name);
    std::fs::read_to_string(file).unwrap()
}

#[test]
fn given_the_device_types_when_exported_then_the_generated_files_have_camel_case_fields() {
    let config = Config::from_env();
    DeviceIdentity::export_all(&config).unwrap();
    Capabilities::export_all(&config).unwrap();
    CalibrationStatus::export_all(&config).unwrap();
    TimeBaseStep::export_all(&config).unwrap();
    VerticalRange::export_all(&config).unwrap();

    let identity = generated("DeviceIdentity.ts");
    assert!(identity.contains("fpgaVersion: string | null"), "{identity}");
    assert!(identity.contains("calibration: CalibrationStatus"), "{identity}");
    let capabilities = generated("Capabilities.ts");
    assert!(
        capabilities.contains("timeBases: Array<TimeBaseStep>"),
        "{capabilities}"
    );
    assert!(capabilities.contains("adcIntervalNs: Array<number>"), "{capabilities}");
    assert!(capabilities.contains("recordLen: number"), "{capabilities}");
    assert!(generated("CalibrationStatus.ts").contains("\"valid\" | \"invalid\""));
}
