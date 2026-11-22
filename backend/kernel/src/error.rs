use std::fmt;

use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../frontend/src/api/generated/")]
pub enum ErrorCategory {
    Recoverable,
    DeviceLost,
    Fatal,
}

/// Error crossing module boundaries. Messages never contain file system paths.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export, export_to = "../../../frontend/src/api/generated/")]
pub struct KernelError {
    pub category: ErrorCategory,
    pub message: String,
}

impl KernelError {
    pub fn new(category: ErrorCategory, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
        }
    }

    pub fn recoverable(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Recoverable, message)
    }

    pub fn fatal(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Fatal, message)
    }

    pub(crate) fn no_handler(request_type: &str) -> Self {
        Self::fatal(format!("no handler registered for request {request_type}"))
    }

    pub(crate) fn duplicate_handler(request_type: &str) -> Self {
        Self::fatal(format!("a handler is already registered for request {request_type}"))
    }
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.category, self.message)
    }
}

impl std::error::Error for KernelError {}
