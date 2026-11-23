use serde::Serialize;
use ts_rs::TS;

use super::DeviceIdentity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Ready,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub enum DeviceSource {
    Hardware,
    Simulator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub enum ConnectionIssueKind {
    PermissionDenied,
    CalibrationInvalid,
    Communication,
}

/// What the user needs to fix a missing USB permission: the shipped rule file and the exact command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub struct PermissionHelp {
    pub rule_file: String,
    pub install_command: String,
    pub doc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub struct ConnectionIssue {
    pub kind: ConnectionIssueKind,
    pub message: String,
    pub help: Option<PermissionHelp>,
}

/// Connection slice (ADR 0013); `revision` is assigned by `device` and starts at 1 per process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub struct ConnectionSnapshot {
    #[ts(type = "number")]
    pub revision: u64,
    pub state: ConnectionState,
    pub source: Option<DeviceSource>,
    pub identity: Option<DeviceIdentity>,
    pub issue: Option<ConnectionIssue>,
    pub simulator_active: bool,
}
