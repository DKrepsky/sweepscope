//! Inert until its own package fills it in: it registers nothing and writes nothing.
use std::sync::Arc;

use sweepscope_kernel::{Bus, Clock, FilePicker, KernelError};

use crate::files::RecordFiles;

pub struct RecordingDeps {
    pub bus: Bus,
    pub clock: Arc<dyn Clock>,
    pub picker: Arc<dyn FilePicker>,
    pub files: Arc<dyn RecordFiles>,
}

pub struct RecordingModule;

impl RecordingModule {
    pub fn new(_deps: RecordingDeps) -> Result<Self, KernelError> {
        Ok(Self)
    }
}
