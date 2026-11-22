use std::fmt;

use sweepscope_kernel::{ErrorCategory, KernelError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceError {
    Disconnected,
    NotFound,
    Timeout,
    Overrun,
    InvalidSetting(String),
    CalibrationInvalid,
    Io(String),
}

impl DeviceError {
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::Disconnected | Self::NotFound => ErrorCategory::DeviceLost,
            _ => ErrorCategory::Recoverable,
        }
    }
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disconnected => f.write_str("the oscilloscope was disconnected"),
            Self::NotFound => f.write_str("the oscilloscope was not found"),
            Self::Timeout => f.write_str("the oscilloscope did not answer in time"),
            Self::Overrun => f.write_str("samples were lost because they were not read in time"),
            Self::InvalidSetting(reason) => write!(f, "invalid setting: {reason}"),
            Self::CalibrationInvalid => f.write_str("the oscilloscope calibration data is invalid"),
            Self::Io(reason) => write!(f, "communication error: {reason}"),
        }
    }
}

impl std::error::Error for DeviceError {}

impl From<DeviceError> for KernelError {
    fn from(error: DeviceError) -> Self {
        KernelError::new(error.category(), error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all() -> Vec<DeviceError> {
        vec![
            DeviceError::Disconnected,
            DeviceError::NotFound,
            DeviceError::Timeout,
            DeviceError::Overrun,
            DeviceError::InvalidSetting("range".into()),
            DeviceError::CalibrationInvalid,
            DeviceError::Io("usb".into()),
        ]
    }

    #[test]
    fn given_a_lost_device_error_when_categorised_then_it_is_device_lost() {
        assert_eq!(DeviceError::Disconnected.category(), ErrorCategory::DeviceLost);
        assert_eq!(DeviceError::NotFound.category(), ErrorCategory::DeviceLost);
    }

    #[test]
    fn given_any_other_device_error_when_categorised_then_it_is_recoverable() {
        for error in all().into_iter().skip(2) {
            assert_eq!(error.category(), ErrorCategory::Recoverable, "{error:?}");
        }
    }

    #[test]
    fn given_each_device_error_when_converted_then_the_kernel_error_keeps_the_category() {
        for error in all() {
            let category = error.category();

            let kernel: KernelError = error.into();

            assert_eq!(kernel.category, category);
        }
    }

    #[test]
    fn given_an_invalid_setting_when_converted_then_the_message_names_the_reason() {
        let kernel: KernelError = DeviceError::InvalidSetting("range index 99".into()).into();

        assert!(kernel.message.contains("range index 99"), "{}", kernel.message);
    }
}
