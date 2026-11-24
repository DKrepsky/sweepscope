use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum CalibrationStatus {
    Valid,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DeviceIdentity {
    pub model: String,
    pub firmware: Option<String>,
    pub fpga_version: Option<String>,
    pub calibration: CalibrationStatus,
    pub simulated: bool,
    /// Stable per-unit id for the zero-offset file; internal, never sent to the frontend.
    #[serde(skip)]
    #[ts(skip)]
    pub unit_id: Option<String>,
}
