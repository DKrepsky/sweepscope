use serde::{Deserialize, Serialize};
use sweepscope_kernel::Request;
use ts_rs::TS;

/// One opaque blob owned by a frontend namespace; settings never interprets `json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UiStateEntry {
    pub namespace: String,
    pub version: u32,
    pub json: String,
}

pub struct LoadUiState {
    pub namespace: String,
}

impl Request for LoadUiState {
    type Response = Option<UiStateEntry>;
}

pub struct SaveUiState(pub UiStateEntry);

impl Request for SaveUiState {
    type Response = ();
}
