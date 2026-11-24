//! A second `ScopeDevice` written against the public traits only runs through `Box<dyn ScopeDevice>` (#3 AC1).
use std::time::Duration;

use sweepscope_device::{
    AcquisitionSetup, CalibrationStatus, Capabilities, ChannelData, DeviceError, DeviceIdentity, Frame, ScopeDevice,
    TriggerOutcome,
};

struct ScriptedDevice {
    identity: DeviceIdentity,
    capabilities: Capabilities,
    captures: u32,
}

impl ScriptedDevice {
    fn new() -> Self {
        Self {
            identity: DeviceIdentity {
                model: "scripted".into(),
                firmware: None,
                fpga_version: None,
                calibration: CalibrationStatus::Valid,
                simulated: true,
                unit_id: None,
            },
            capabilities: Capabilities {
                channel_count: 1,
                record_len: 2,
                time_bases: vec![],
                vertical_ranges: vec![],
                adc_interval_ns: vec![1.0],
                samples_per_div: 250.0,
            },
            captures: 0,
        }
    }

    fn frame(&self) -> Frame {
        let data = ChannelData {
            codes: vec![1, 2],
            volts_per_div: 1.0,
            zero_code: 128.0,
        };
        Frame {
            channels: [Some(data), None, None, None],
            counts_per_div: 24.0,
            sample_interval_s: 1e-9,
            trigger: TriggerOutcome::None,
        }
    }
}

impl ScopeDevice for ScriptedDevice {
    fn identity(&self) -> &DeviceIdentity {
        &self.identity
    }
    fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }
    fn configure(&mut self, _: &AcquisitionSetup) -> Result<(), DeviceError> {
        Ok(())
    }
    fn capture(&mut self, _: Duration) -> Result<Frame, DeviceError> {
        self.captures += 1;
        match self.captures {
            1 => Ok(self.frame()),
            _ => Err(DeviceError::Timeout),
        }
    }
    fn force_capture(&mut self, _: Duration) -> Result<Frame, DeviceError> {
        Ok(self.frame())
    }
    fn start_roll(&mut self) -> Result<(), DeviceError> {
        Ok(())
    }
    fn poll_roll(&mut self) -> Result<Frame, DeviceError> {
        Err(DeviceError::Overrun)
    }
    fn stop(&mut self) -> Result<(), DeviceError> {
        Ok(())
    }
}

#[test]
fn given_a_second_device_implemented_in_a_test_when_used_as_a_boxed_trait_object_then_it_captures_and_reports_errors() {
    let mut device: Box<dyn ScopeDevice> = Box::new(ScriptedDevice::new());

    let first = device.capture(Duration::from_millis(1));
    let second = device.capture(Duration::from_millis(1));

    assert_eq!(first.unwrap().channels[0].as_ref().unwrap().codes, vec![1, 2]);
    assert_eq!(second.err(), Some(DeviceError::Timeout));
    assert_eq!(device.identity().model, "scripted");
    assert_eq!(device.poll_roll().err(), Some(DeviceError::Overrun));
}
