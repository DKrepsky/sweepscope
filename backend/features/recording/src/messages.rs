use serde::Serialize;
use sweepscope_kernel::contracts::{FormattedValue, SavedFile};
use sweepscope_kernel::{CommandDescriptor, Event, Request};
use ts_rs::TS;

/// Why the last recording stopped on its own; the UI shows each `id` once.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingFailure {
    #[ts(type = "number")]
    pub id: u64,
    pub text: String,
    pub file_name: String,
    #[ts(type = "number")]
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingState {
    #[ts(type = "number")]
    pub revision: u64,
    pub active: bool,
    pub file_name: Option<String>,
    pub duration: FormattedValue,
    pub bytes_written: FormattedValue,
    pub failure: Option<RecordingFailure>,
}

/// Asks for a file name and starts writing; `None` when the dialog is cancelled. Rejected outside a roll session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartRecording;
impl Request for StartRecording {
    type Response = Option<SavedFile>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StopRecording;
impl Request for StopRecording {
    type Response = ();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetRecording;
impl Request for GetRecording {
    type Response = RecordingState;
}

/// Published at most once a second; the shell forwards it as `recording_changed`.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordingChanged(pub RecordingState);
impl Event for RecordingChanged {}

/// Frontend-callable commands of this feature, consumed by the shell to generate `commands.ts`.
pub const COMMANDS: &[CommandDescriptor] = &[
    CommandDescriptor {
        name: "start_recording",
        args: "{}",
        result: "SavedFile | null",
        types: &["SavedFile"],
    },
    CommandDescriptor {
        name: "stop_recording",
        args: "{}",
        result: "null",
        types: &[],
    },
];
