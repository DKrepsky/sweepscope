//! Writes the roll items to a `.ssroll` file while the user records (ADR 0017).
#![forbid(unsafe_code)]

mod files;
mod format;
mod messages;
mod module;
mod os_files;
mod recorder;

pub use files::{RecordFiles, RecordWriter};
pub use messages::{
    GetRecording, RecordingChanged, RecordingFailure, RecordingState, StartRecording, StopRecording, COMMANDS,
};
pub use module::{RecordingDeps, RecordingModule};
pub use os_files::OsRecordFiles;
