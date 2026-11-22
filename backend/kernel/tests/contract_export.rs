use std::path::Path;

use sweepscope_kernel::contracts::{
    CalibrationStatus, Capabilities, ConnectionIssue, ConnectionIssueKind, ConnectionSnapshot, ConnectionState,
    Coupling, DeviceIdentity, DeviceSource, PermissionHelp, Slope, Sweep, TimeBaseStep, VerticalRange,
};
use sweepscope_kernel::KernelError;
use ts_rs::{Config, TS};

fn generated(name: &str) -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../frontend/src/api/generated")
        .join(name);
    std::fs::read_to_string(file).unwrap()
}

#[test]
fn given_the_kernel_error_when_exported_then_the_generated_file_has_category_and_message() {
    KernelError::export_all(&Config::from_env()).unwrap();

    let text = generated("KernelError.ts");
    assert!(
        text.contains("category: ErrorCategory") && text.contains("message: string"),
        "{text}"
    );
}

#[test]
fn given_the_device_types_when_exported_then_the_generated_files_have_camel_case_fields() {
    let config = Config::from_env();
    DeviceIdentity::export_all(&config).unwrap();
    Capabilities::export_all(&config).unwrap();
    CalibrationStatus::export_all(&config).unwrap();
    TimeBaseStep::export_all(&config).unwrap();
    VerticalRange::export_all(&config).unwrap();

    assert!(generated("DeviceIdentity.ts").contains("fpgaVersion: string | null"));
    let capabilities = generated("Capabilities.ts");
    assert!(
        capabilities.contains("timeBases: Array<TimeBaseStep>"),
        "{capabilities}"
    );
    assert!(capabilities.contains("adcIntervalNs: Array<number>"), "{capabilities}");
    assert!(capabilities.contains("recordLen: number"), "{capabilities}");
    assert!(generated("CalibrationStatus.ts").contains("\"valid\" | \"invalid\""));
    assert!(generated("VerticalRange.ts").contains("maxPositionDivs: number"));
}

#[test]
fn given_the_connection_slice_when_exported_then_the_revision_is_a_number_and_options_are_nullable() {
    let config = Config::from_env();
    ConnectionSnapshot::export_all(&config).unwrap();
    ConnectionIssue::export_all(&config).unwrap();
    PermissionHelp::export_all(&config).unwrap();
    ConnectionIssueKind::export_all(&config).unwrap();
    ConnectionState::export_all(&config).unwrap();
    DeviceSource::export_all(&config).unwrap();

    let snapshot = generated("ConnectionSnapshot.ts");
    assert!(snapshot.contains("revision: number"), "{snapshot}");
    assert!(snapshot.contains("simulatorActive: boolean"), "{snapshot}");
    assert!(snapshot.contains("source: DeviceSource | null"), "{snapshot}");
    assert!(generated("ConnectionState.ts").contains("\"disconnected\" | \"connecting\" | \"ready\" | \"error\""));
    assert!(generated("ConnectionIssueKind.ts").contains("\"permissionDenied\""));
    assert!(generated("PermissionHelp.ts").contains("installCommand: string"));
}

#[test]
fn given_the_setup_enums_when_exported_then_they_are_lower_camel_strings() {
    let config = Config::from_env();
    Coupling::export_all(&config).unwrap();
    Slope::export_all(&config).unwrap();
    Sweep::export_all(&config).unwrap();

    assert!(generated("Coupling.ts").contains("\"ac\" | \"dc\""));
    assert!(generated("Slope.ts").contains("\"rising\" | \"falling\""));
    assert!(generated("Sweep.ts").contains("\"auto\" | \"normal\""));
}
