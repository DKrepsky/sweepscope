use std::fmt;

/// The process exit codes of ADR 0022 item 6.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    Success,
    Failure,
    Usage,
    NoDevice,
    Timeout,
    PermissionDenied,
}

impl ExitStatus {
    pub fn code(self) -> u8 {
        match self {
            Self::Success => 0,
            Self::Failure => 1,
            Self::Usage => 2,
            Self::NoDevice => 10,
            Self::Timeout => 11,
            Self::PermissionDenied => 12,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliError {
    pub status: ExitStatus,
    pub message: String,
}

impl CliError {
    pub fn new(status: ExitStatus, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self::new(ExitStatus::Failure, message)
    }

    /// An I/O failure on a named file; a missing permission has its own exit code.
    pub fn io(action: &str, file: &str, error: &std::io::Error) -> Self {
        let status = if error.kind() == std::io::ErrorKind::PermissionDenied {
            ExitStatus::PermissionDenied
        } else {
            ExitStatus::Failure
        };
        Self::new(status, format!("cannot {action} {file}: {error}"))
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Error, ErrorKind};

    use super::*;

    #[test]
    fn given_each_status_when_its_code_is_read_then_it_matches_the_documented_exit_code() {
        let codes = [
            ExitStatus::Success,
            ExitStatus::Failure,
            ExitStatus::Usage,
            ExitStatus::NoDevice,
            ExitStatus::Timeout,
            ExitStatus::PermissionDenied,
        ]
        .map(ExitStatus::code);

        assert_eq!(codes, [0, 1, 2, 10, 11, 12]);
    }

    #[test]
    fn given_a_permission_error_when_converted_then_the_status_is_permission_denied() {
        let error = CliError::io("write", "a.csv", &Error::from(ErrorKind::PermissionDenied));

        assert_eq!(error.status, ExitStatus::PermissionDenied);
    }

    #[test]
    fn given_any_other_io_error_when_converted_then_the_status_is_failure() {
        let error = CliError::io("write", "a.csv", &Error::from(ErrorKind::NotFound));

        assert_eq!(error.status, ExitStatus::Failure);
    }
}
