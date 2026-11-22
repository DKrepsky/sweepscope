use sweepscope_kernel::contracts::{ConnectionIssue, ConnectionIssueKind, PermissionHelp};

use crate::DeviceError;

const RULE_FILE: &str = "packaging/udev/99-hantek6xx4.rules";
const DOC: &str = "docs/usb-permissions.md";

/// The command installs the shipped rule file; `uaccess` in it grants the logged-in user access without a group change.
fn permission_help() -> PermissionHelp {
    PermissionHelp {
        rule_file: RULE_FILE.to_owned(),
        install_command: format!(
            "sudo cp {RULE_FILE} /etc/udev/rules.d/ && sudo udevadm control --reload-rules && sudo udevadm trigger"
        ),
        doc: DOC.to_owned(),
    }
}

/// The banner a failed open or a failed capture shows. `None` means the device simply went away: no banner.
pub fn issue_for(error: &DeviceError) -> Option<ConnectionIssue> {
    let (kind, help) = match error {
        DeviceError::Disconnected | DeviceError::NotFound => return None,
        DeviceError::PermissionDenied => (ConnectionIssueKind::PermissionDenied, Some(permission_help())),
        DeviceError::CalibrationInvalid => (ConnectionIssueKind::CalibrationInvalid, None),
        _ => (ConnectionIssueKind::Communication, None),
    };
    Some(ConnectionIssue {
        kind,
        message: error.to_string(),
        help,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_a_device_that_went_away_when_the_issue_is_asked_then_there_is_none() {
        assert_eq!(issue_for(&DeviceError::NotFound), None);
        assert_eq!(issue_for(&DeviceError::Disconnected), None);
    }

    #[test]
    fn given_a_permission_error_when_the_issue_is_asked_then_it_carries_the_install_command() {
        let issue = issue_for(&DeviceError::PermissionDenied).unwrap();

        let help = issue.help.unwrap();
        assert_eq!(issue.kind, ConnectionIssueKind::PermissionDenied);
        assert!(help.install_command.contains(&help.rule_file), "{help:?}");
        assert!(help.install_command.contains("/etc/udev/rules.d/"), "{help:?}");
        assert_eq!(help.doc, "docs/usb-permissions.md");
    }

    #[test]
    fn given_invalid_calibration_when_the_issue_is_asked_then_the_kind_says_so_without_help() {
        let issue = issue_for(&DeviceError::CalibrationInvalid).unwrap();

        assert_eq!(issue.kind, ConnectionIssueKind::CalibrationInvalid);
        assert_eq!(issue.help, None);
    }

    #[test]
    fn given_an_io_error_when_the_issue_is_asked_then_it_is_a_communication_issue() {
        let issue = issue_for(&DeviceError::Io("usb".into())).unwrap();

        assert_eq!(issue.kind, ConnectionIssueKind::Communication);
        assert!(issue.message.contains("usb"));
    }
}
