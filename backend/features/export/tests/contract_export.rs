use std::path::Path;

use sweepscope_export::ExportFormat;
use ts_rs::{Config, TS};

#[test]
fn given_the_export_format_when_exported_then_it_is_a_lower_camel_string() {
    ExportFormat::export_all(&Config::from_env()).unwrap();

    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../frontend/src/api/generated/ExportFormat.ts");
    assert!(std::fs::read_to_string(file).unwrap().contains("\"csv\" | \"raw\""));
}
