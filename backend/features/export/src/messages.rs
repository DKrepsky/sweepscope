use serde::Deserialize;
use sweepscope_kernel::contracts::SavedFile;
use sweepscope_kernel::{CommandDescriptor, Request};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ExportFormat {
    Csv,
    Raw,
}

/// Exports the newest frame; `None` when the dialog is cancelled; Recoverable `No waveform to export` before the first frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportFrame {
    pub format: ExportFormat,
}
impl Request for ExportFrame {
    type Response = Option<SavedFile>;
}

/// Saves PNG bytes composed by the frontend; Recoverable when the body is not a PNG or exceeds 32 MiB.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveScreenshot {
    pub png: Vec<u8>,
}
impl Request for SaveScreenshot {
    type Response = Option<SavedFile>;
}

pub const COMMANDS: &[CommandDescriptor] = &[
    CommandDescriptor {
        name: "export_frame",
        args: "{ format: ExportFormat }",
        result: "SavedFile | null",
        types: &["ExportFormat", "SavedFile"],
    },
    CommandDescriptor {
        name: "save_screenshot",
        args: "Uint8Array",
        result: "SavedFile | null",
        types: &["SavedFile"],
    },
];
