use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../frontend/src/api/generated/")]
pub enum CalibrationStatus {
    Valid,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../frontend/src/api/generated/")]
pub struct DeviceIdentity {
    pub model: String,
    pub firmware: Option<String>,
    pub fpga_version: Option<String>,
    pub calibration: CalibrationStatus,
    pub simulated: bool,
}
