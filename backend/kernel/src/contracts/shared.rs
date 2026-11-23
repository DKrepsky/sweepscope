use serde::Serialize;
use ts_rs::TS;

/// A number for geometry (`value`, SI) next to what the UI shows: the number part and the unit with prefix.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub struct FormattedValue {
    pub value: f64,
    pub text: String,
    pub unit: String,
}

/// Reply of every command that writes a file the user chose. Never holds a path (ADR 0016 item 2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub struct SavedFile {
    pub file_name: String,
}
