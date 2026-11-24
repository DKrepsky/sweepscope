use std::time::Duration;

use hantek6xx4_lib::{CaptureOptions, Scope, Transport};

use super::convert::{capture_frame, config_from_setup, device_error, roll_frame};
use crate::profile::capabilities;
use crate::{
    AcquisitionSetup, CalibrationStatus, Capabilities, DeviceError, DeviceIdentity, Frame, ScopeDevice, Sweep,
};

/// One opened Hantek scope. The driver has no force trigger, so `force_capture` captures with the auto sweep and the
/// requested sweep is re-applied before the next normal capture (ADR 0012 item 6).
pub struct HantekDevice<T: Transport> {
    pub(super) scope: Scope<T>,
    identity: DeviceIdentity,
    capabilities: Capabilities,
    requested: Option<AcquisitionSetup>,
    needs_restore: bool,
}

impl<T: Transport> HantekDevice<T> {
    /// Initialises the scope on `transport`; fails with `CalibrationInvalid` when the device's tables are bad.
    pub fn new(transport: T) -> Result<Self, DeviceError> {
        let mut scope = Scope::new(transport).map_err(device_error)?;
        let identity = identity_of(&mut scope);
        Ok(Self {
            scope,
            identity,
            capabilities: capabilities(),
            requested: None,
            needs_restore: false,
        })
    }

    fn requested(&self) -> Result<&AcquisitionSetup, DeviceError> {
        self.requested
            .as_ref()
            .ok_or_else(|| DeviceError::InvalidSetting("the device is not configured".to_owned()))
    }

    fn prepare_acquisition(&mut self) -> Result<(), DeviceError> {
        let setup = self.requested()?.clone();
        if self.needs_restore {
            self.apply(&setup)?;
        }
        Ok(())
    }

    fn apply(&mut self, setup: &AcquisitionSetup) -> Result<(), DeviceError> {
        let config = config_from_setup(setup)?;
        self.scope.configure(&config).map_err(device_error)?;
        self.needs_restore = false;
        Ok(())
    }

    fn capture_with(&mut self, timeout: Duration) -> Result<Frame, DeviceError> {
        let frame = self.scope.capture(CaptureOptions { timeout }).map_err(device_error)?;
        Ok(capture_frame(&frame))
    }
}

fn identity_of<T: Transport>(scope: &mut Scope<T>) -> DeviceIdentity {
    let firmware = scope
        .driver_version()
        .map(|version| format!("driver version {version}"));
    let info = scope.info();
    DeviceIdentity {
        model: format!("Hantek 6xx4 (family {}), PCB {}", info.name, info.pcb_version),
        firmware,
        fpga_version: Some(format!("0x{:04X}", info.fpga_version)),
        calibration: CalibrationStatus::Valid,
        simulated: false,
        unit_id: unit_id(&info.name, &info.pcb_version, &info.serial),
    }
}

/// `None` when the vendor's serial-like field is empty, so two blank units never share an offset file.
fn unit_id(family: &str, pcb: &str, serial: &str) -> Option<String> {
    let serial = serial.trim_matches(|c: char| c.is_whitespace() || c == '\0');
    (!serial.is_empty()).then(|| format!("{family}/{pcb}/{serial}"))
}

impl<T: Transport + Send> ScopeDevice for HantekDevice<T> {
    fn identity(&self) -> &DeviceIdentity {
        &self.identity
    }

    fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    fn configure(&mut self, setup: &AcquisitionSetup) -> Result<(), DeviceError> {
        self.apply(setup)?;
        self.requested = Some(setup.clone());
        Ok(())
    }

    fn capture(&mut self, timeout: Duration) -> Result<Frame, DeviceError> {
        self.prepare_acquisition()?;
        self.capture_with(timeout)
    }

    fn force_capture(&mut self, timeout: Duration) -> Result<Frame, DeviceError> {
        let requested = self.requested()?;
        if requested.trigger.sweep == Sweep::Normal {
            let mut free_run = requested.clone();
            free_run.trigger.sweep = Sweep::Auto;
            let applied = self.apply(&free_run);
            self.needs_restore = true;
            applied?;
        }
        self.capture_with(timeout)
    }

    fn start_roll(&mut self) -> Result<(), DeviceError> {
        self.prepare_acquisition()?;
        self.scope.start_roll().map_err(device_error)
    }

    fn poll_roll(&mut self) -> Result<Frame, DeviceError> {
        let frame = self.scope.poll_roll().map_err(device_error)?;
        Ok(roll_frame(&frame))
    }

    fn stop(&mut self) -> Result<(), DeviceError> {
        self.scope.stop().map_err(device_error)
    }
}

#[cfg(test)]
mod tests {
    use super::unit_id;

    #[test]
    fn given_family_pcb_and_serial_when_the_unit_id_is_built_then_they_are_joined_with_slashes() {
        assert_eq!(unit_id("6000", "P1.06", "ABC123").as_deref(), Some("6000/P1.06/ABC123"));
    }

    #[test]
    fn given_a_serial_with_padding_when_the_unit_id_is_built_then_the_padding_is_trimmed() {
        assert_eq!(
            unit_id("6000", "P1.06", "  ABC123 \0\0").as_deref(),
            Some("6000/P1.06/ABC123")
        );
    }

    #[test]
    fn given_an_empty_or_blank_serial_when_the_unit_id_is_built_then_there_is_none() {
        assert_eq!(unit_id("6000", "P1.06", ""), None);
        assert_eq!(unit_id("6000", "P1.06", "   \0"), None);
    }
}
