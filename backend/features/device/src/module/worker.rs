use std::sync::Arc;
use std::time::Duration;

use sweepscope_kernel::contracts::{CaptureCompleted, CaptureOutcome, RollPolled};
use sweepscope_kernel::{Bus, Clock};

use super::control::{CaptureKind, Work, WorkerControl};
use super::roll::{RollCounter, POLL_INTERVAL};
use super::timeout::{capture_timeout, sample_interval_s};
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
    setup: AcquisitionSetup,
    timeout: Duration,
    sample_interval_s: f64,
    streaming: bool,
    /// The device was configured with this setup and has not been touched since.
    fresh: bool,
}

/// The one thread that talks to an opened device. Serial access, no lock held across a device call.
pub struct Worker {
    pub device: Box<dyn ScopeDevice>,
    pub control: Arc<WorkerControl>,
    pub bus: Bus,
    pub clock: Arc<dyn Clock>,
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
                    let (Some(kind), Some(applied)) = (capture, applied.as_mut()) else {
                        continue;
                    };
                    let result = if applied.streaming {
                        self.roll(applied)
                    } else {
                        self.capture(kind, applied)
                    };
                    if let Err(error) = result {
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
        let capabilities = self.device.capabilities();
        let streaming = capabilities
            .time_bases
            .get(usize::from(setup.time_base_index))
            .is_some_and(|step| step.streaming);
        Ok(Applied {
            revision,
            setup: setup.clone(),
            timeout: capture_timeout(capabilities, setup),
            sample_interval_s: sample_interval_s(capabilities, setup),
            streaming,
            fresh: true,
        })
    }

    /// Rolls until stop, a new setup or capturing off: stop, configure, start, then poll. Items of an ended roll are never published.
    fn roll(&mut self, applied: &mut Applied) -> Result<(), DeviceError> {
        if !std::mem::take(&mut applied.fresh) {
            self.device.stop()?;
            self.device.configure(&applied.setup)?;
        }
        self.device.start_roll()?;
        let mut counter = RollCounter::new(self.clock.now(), applied.sample_interval_s);
        while !self.control.roll_should_end_within(POLL_INTERVAL) {
            let item = match self.device.poll_roll() {
                Ok(frame) => counter.samples(self.clock.now(), frame),
                Err(DeviceError::Overrun) => Some(counter.gap(self.clock.now())),
                Err(error) => return Err(error),
            };
            if let Some(item) = item.filter(|_| self.control.is_capturing()) {
                self.bus.publish(RollPolled {
                    setup_revision: applied.revision,
                    item,
                });
            }
        }
        self.device.stop()
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
