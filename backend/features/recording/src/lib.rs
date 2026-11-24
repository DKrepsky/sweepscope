//! Writes the roll items to a `.ssroll` file while the user records (ADR 0017).
#![forbid(unsafe_code)]

mod files;
mod messages;
mod module;

pub use files::{RecordFiles, RecordWriter};
pub use messages::{GetRecording, RecordingChanged, RecordingFailure, RecordingState, StartRecording, StopRecording};
pub use module::{RecordingDeps, RecordingModule};
