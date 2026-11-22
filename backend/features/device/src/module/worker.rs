use std::sync::Arc;
use std::time::Duration;

use sweepscope_kernel::contracts::{CaptureCompleted, CaptureOutcome};
use sweepscope_kernel::Bus;

use super::control::{CaptureKind, Work, WorkerControl};
use super::timeout::capture_timeout;
use crate::{AcquisitionSetup, DeviceError, ScopeDevice};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerEnd {
    /// The instrument is gone; the worker has ended.
    Disconnected,
    /// The instrument answered with an error; capturing stopped, the worker keeps serving setups.
    Failed(DeviceError),
}

struct Applied {
    revision: u64,
    timeout: Duration,
}

/// The one thread that talks to an opened device. Serial access, no lock held across a device call.
pub struct Worker {
    pub device: Box<dyn ScopeDevice>,
    pub control: Arc<WorkerControl>,
    pub bus: Bus,
    pub report: Box<dyn Fn(WorkerEnd) + Send>,
}

impl Worker {
    pub fn run(mut self) {
        let mut applied: Option<Applied> = None;
        loop {
            match self.control.next_work(applied.is_some()) {
                Work::Stop => return self.shut_down(),
                Work::Run { setup, capture } => {
                    if let Some((revision, setup)) = setup {
                        match self.configure(revision, &setup) {
                            Ok(done) => applied = Some(done),
                            Err(error) => {
                                if self.fail(error) {
                                    return;
                                }
                                continue;
                            }
                        }
                    }
                    let (Some(kind), Some(applied)) = (capture, applied.as_ref()) else {
                        continue;
                    };
                    if let Err(error) = self.capture(kind, applied) {
                        if self.fail(error) {
                            return;
                        }
                    }
                }
            }
        }
    }

    fn configure(&mut self, revision: u64, setup: &AcquisitionSetup) -> Result<Applied, DeviceError> {
        self.device.configure(setup)?;
        Ok(Applied {
            revision,
            timeout: capture_timeout(self.device.capabilities(), setup),
        })
    }

    fn capture(&mut self, kind: CaptureKind, applied: &Applied) -> Result<(), DeviceError> {
        let result = match kind {
            CaptureKind::Normal => self.device.capture(applied.timeout),
            CaptureKind::Forced => self.device.force_capture(applied.timeout),
        };
        let outcome = match result {
            Ok(frame) => CaptureOutcome::Frame(Arc::new(frame)),
            Err(DeviceError::Timeout) => CaptureOutcome::TimedOut,
            Err(error) => return Err(error),
        };
        if self.control.is_capturing() {
            self.bus.publish(CaptureCompleted {
                setup_revision: applied.revision,
                outcome,
            });
        }
        Ok(())
    }

    /// Reports the error; returns true when the worker must end.
    fn fail(&mut self, error: DeviceError) -> bool {
        let lost = matches!(error, DeviceError::Disconnected | DeviceError::NotFound);
        if lost {
            (self.report)(WorkerEnd::Disconnected);
        } else {
            self.control.set_capturing(false);
            (self.report)(WorkerEnd::Failed(error));
        }
        lost
    }

    fn shut_down(&mut self) {
        let _ = self.device.stop();
    }
}
