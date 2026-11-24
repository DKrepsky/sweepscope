//! The module's state behind one lock: connection, offsets file, the open session and the slice.
use std::sync::Arc;
use std::time::{Duration, Instant};

use sweepscope_kernel::contracts::{
    Availability, Capabilities, CaptureProbe, ConnectionChanged, ConnectionState, HoldInstrument, ProbeCaptured,
    ZeroOffsetsChanged,
};
use sweepscope_kernel::{Bus, Clock, KernelError, Storage};

use super::probe::probe_setup;
use super::stamp::iso_8601;
use crate::domain::offsets::{OffsetFile, StoredUnit};
use crate::domain::rules::{CHANNEL_TIMEOUT, PROBE_DISCARD_FRAMES, PROBE_KEEP_FRAMES, PROBE_TIMEOUT};
use crate::domain::session::{Accepted, SessionMachine};
use crate::domain::statistics::Reading;
use crate::domain::sweep::{MeasurePlan, Step, ZeroSweep};
use crate::domain::texts;
use crate::domain::view::{unit_calibration, Slice};
use crate::{ZeroCalibrationChanged, ZeroCalibrationState, ZeroStep};

const HOLD_REASON: &str = "Zero calibration is open";
const REFUSAL_SHOWN: Duration = Duration::from_secs(3);

const DEVICE_LOST: &str = "Device disconnected. Calibration cancelled. Nothing was stored.";
const PROBE_FAILED: &str = "The scope delivered no data for the measurement. Nothing was stored.";
const PROBE_TIMED_OUT: &str = "The scope did not answer in time. Nothing was stored.";
const CHANNEL_TIMED_OUT: &str = "The channel took too long to measure. Nothing was stored.";
const NO_PROBE_SETUP: &str = "The scope offers no usable time base. Nothing was stored.";
const NO_SESSION: &str = "Zero calibration is not open";
const WRONG_STEP: &str = "That is not possible at this step of the zero calibration";

pub struct Environment {
    pub bus: Bus,
    pub clock: Arc<dyn Clock>,
    pub storage: Arc<dyn Storage>,
    pub unix_time_us: fn() -> f64,
}

struct Scope {
    capabilities: Capabilities,
    ranges: Vec<f64>,
    unit_id: Option<String>,
}

enum Link {
    Down(&'static str),
    Ready(Scope),
}

struct Awaited {
    id: u64,
    channel: usize,
    deadline: Instant,
}

struct Active {
    machine: SessionMachine,
    sweep: Option<ZeroSweep>,
    probe: Option<Awaited>,
    channel_deadline: Option<Instant>,
}

impl Active {
    fn stop_measuring(&mut self) {
        self.sweep = None;
        self.probe = None;
        self.channel_deadline = None;
    }
}

struct Refusal {
    reason: String,
    until: Instant,
}

pub struct Core {
    env: Environment,
    link: Link,
    file: OffsetFile,
    /// Outer `None`: nothing announced for this connection; inner `None`: announced "no offsets".
    announced: Option<Option<String>>,
    slice: Slice,
    session: Option<Active>,
    refusal: Option<Refusal>,
    next_probe_id: u64,
}

impl Core {
    pub fn new(env: Environment) -> Self {
        Self {
            env,
            link: Link::Down(texts::NO_DEVICE),
            file: OffsetFile::default(),
            announced: None,
            slice: Slice::new(texts::unavailable(texts::NO_DEVICE)),
            session: None,
            refusal: None,
            next_probe_id: 1,
        }
    }

    pub fn state(&self) -> ZeroCalibrationState {
        self.slice.state()
    }

    // Lock invariant (open, close, request_probe): the module lock is held across `bus.request`. Safe because the
    // acquisition handlers for Hold/CaptureProbe return at once and never call back into this module.
    pub fn open(&mut self) -> Result<(), KernelError> {
        if self.session.is_some() {
            return Ok(());
        }
        self.require_available()?;
        let Link::Ready(scope) = &self.link else {
            return Err(KernelError::recoverable(texts::NEEDS_READY_SCOPE));
        };
        let machine = SessionMachine::new(scope.ranges.clone());
        let hold = HoldInstrument {
            on: true,
            reason: HOLD_REASON.to_owned(),
        };
        if let Err(refused) = self.env.bus.request(hold) {
            self.refuse(&refused.message);
            return Err(refused);
        }
        self.session = Some(Active {
            machine,
            sweep: None,
            probe: None,
            channel_deadline: None,
        });
        self.refresh();
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), KernelError> {
        self.active()?.machine.start().map_err(wrong_step)?;
        self.refresh();
        Ok(())
    }

    pub fn measure(&mut self) -> Result<(), KernelError> {
        let now = self.env.clock.now();
        let ranges = self.ready_ranges()?;
        let active = self.active()?;
        let channel = active.machine.measure().map_err(wrong_step)?;
        let mut sweep = ZeroSweep::new(ranges);
        let step = sweep.next(None);
        active.sweep = Some(sweep);
        active.channel_deadline = Some(now + CHANNEL_TIMEOUT);
        self.apply(step, channel);
        self.refresh();
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), KernelError> {
        let Some(mut active) = self.session.take() else {
            return Ok(());
        };
        active.machine.cancel();
        let release = self.env.bus.request(HoldInstrument {
            on: false,
            reason: HOLD_REASON.to_owned(),
        });
        self.refresh();
        release
    }

    pub fn connection(&mut self, change: ConnectionChanged) {
        let ready = change.snapshot.state == ConnectionState::Ready;
        match change.capabilities.filter(|_| ready) {
            Some(capabilities) => {
                let unit_id = change.snapshot.identity.and_then(|identity| identity.unit_id);
                self.scope_ready(capabilities, unit_id);
            }
            None => self.scope_down(change.snapshot.state),
        }
        self.refresh();
    }

    pub fn probe_captured(&mut self, probe: ProbeCaptured) {
        let Some(awaited) = self
            .session
            .as_mut()
            .and_then(|a| a.probe.take_if(|p| p.id == probe.id))
        else {
            return;
        };
        let reading = Reading::from_frames(&probe.frames, awaited.channel);
        let step = match (self.session.as_mut().and_then(|a| a.sweep.as_mut()), reading) {
            (Some(sweep), Some(reading)) => sweep.next(Some(&reading)),
            _ => {
                self.fail(PROBE_FAILED);
                self.refresh();
                return;
            }
        };
        self.apply(step, awaited.channel);
        self.refresh();
    }

    /// Looks at the deadlines; the thread calls it between events.
    pub fn tick(&mut self) {
        let now = self.env.clock.now();
        let mut changed = self.refusal.take_if(|r| now >= r.until).is_some();
        let expired = self.session.as_ref().and_then(|a| {
            if a.channel_deadline.is_some_and(|d| now >= d) {
                Some(CHANNEL_TIMED_OUT)
            } else if a.probe.as_ref().is_some_and(|p| now >= p.deadline) {
                Some(PROBE_TIMED_OUT)
            } else {
                None
            }
        });
        if let Some(text) = expired {
            self.fail(text);
            changed = true;
        }
        if changed {
            self.refresh();
        }
    }

    fn scope_ready(&mut self, capabilities: Capabilities, unit_id: Option<String>) {
        let ranges = capabilities.vertical_ranges.iter().map(|r| r.volts_per_div).collect();
        let first_time = self.announced.as_ref() != Some(&unit_id);
        self.link = Link::Ready(Scope {
            capabilities,
            ranges,
            unit_id: unit_id.clone(),
        });
        if first_time {
            self.file = OffsetFile::load(&*self.env.storage).unwrap_or_default();
            self.announce(unit_id);
        }
    }

    fn scope_down(&mut self, state: ConnectionState) {
        let reason = match state {
            ConnectionState::Disconnected => texts::NO_DEVICE,
            _ => texts::NEEDS_READY_SCOPE,
        };
        self.link = Link::Down(reason);
        self.announced = None;
        let unfinished = self
            .session
            .as_ref()
            .is_some_and(|a| !matches!(a.machine.step(), ZeroStep::Done | ZeroStep::Failed));
        if unfinished {
            self.fail(DEVICE_LOST);
        }
    }

    fn announce(&mut self, unit_id: Option<String>) {
        let offsets = unit_id.as_deref().and_then(|id| self.file.offsets_for(id));
        self.env.bus.publish(ZeroOffsetsChanged { offsets });
        self.announced = Some(unit_id);
    }

    fn apply(&mut self, step: Step, channel: usize) {
        match step {
            Step::Measure(plan) => self.request_probe(plan, channel),
            Step::NotShorted(rejection) => {
                if let Some(active) = self.session.as_mut() {
                    let _ = active.machine.reject(vec![rejection.line]);
                    active.stop_measuring();
                }
            }
            Step::Done(offsets) => self.accept_channel(offsets),
        }
    }

    fn request_probe(&mut self, plan: MeasurePlan, channel: usize) {
        let Link::Ready(scope) = &self.link else {
            return;
        };
        let Some(setup) = probe_setup(&scope.capabilities, plan.range_index, channel) else {
            self.fail(NO_PROBE_SETUP);
            return;
        };
        let id = self.next_probe_id;
        self.next_probe_id += 1;
        let request = CaptureProbe {
            id,
            setup,
            discard: PROBE_DISCARD_FRAMES,
            keep: PROBE_KEEP_FRAMES,
        };
        if let Err(error) = self.env.bus.request(request) {
            self.fail(&format!("{} Nothing was stored.", error.message));
            return;
        }
        let deadline = self.env.clock.now() + PROBE_TIMEOUT;
        if let Some(active) = self.session.as_mut() {
            active
                .machine
                .set_progress(active.sweep.as_ref().map_or(0.0, ZeroSweep::progress));
            active.probe = Some(Awaited { id, channel, deadline });
        }
    }

    fn accept_channel(&mut self, offsets: Vec<f64>) {
        let Some(active) = self.session.as_mut() else {
            return;
        };
        active.stop_measuring();
        match active.machine.accept(offsets) {
            Ok(Accepted::Complete(volts)) => self.commit(volts),
            Ok(Accepted::NextChannel) | Err(_) => {}
        }
    }

    /// One write of all four channels, then the announcement; a failed write stores and announces nothing.
    fn commit(&mut self, volts: [Vec<f64>; 4]) {
        let Link::Ready(Scope {
            unit_id: Some(unit_id), ..
        }) = &self.link
        else {
            self.fail(DEVICE_LOST);
            return;
        };
        let unit_id = unit_id.clone();
        let stored = StoredUnit {
            stored_at_utc: iso_8601((self.env.unix_time_us)()),
            volts,
        };
        let storage = &*self.env.storage;
        let saved = OffsetFile::load(storage).and_then(|file| {
            let file = file.with_unit(&unit_id, stored);
            file.save(storage).map(|()| file)
        });
        match saved {
            Ok(file) => {
                self.file = file;
                if let Some(active) = self.session.as_mut() {
                    let _ = active.machine.committed();
                }
                self.announce(Some(unit_id));
            }
            Err(error) => self.fail(&format!(
                "The offsets could not be saved: {}. Nothing was stored.",
                error.message
            )),
        }
    }

    fn fail(&mut self, text: &str) {
        if let Some(active) = self.session.as_mut() {
            active.machine.fail(text);
            active.stop_measuring();
        }
    }

    fn refuse(&mut self, reason: &str) {
        self.refusal = Some(Refusal {
            reason: reason.to_owned(),
            until: self.env.clock.now() + REFUSAL_SHOWN,
        });
        self.refresh();
    }

    fn require_available(&self) -> Result<(), KernelError> {
        let availability = self.availability();
        if availability.enabled {
            return Ok(());
        }
        Err(KernelError::recoverable(availability.reason.unwrap_or_default()))
    }

    fn availability(&self) -> Availability {
        if let Some(refusal) = &self.refusal {
            return texts::unavailable(&refusal.reason);
        }
        match &self.link {
            Link::Down(reason) => texts::unavailable(reason),
            Link::Ready(Scope { unit_id: None, .. }) => texts::unavailable(texts::NO_SERIAL),
            Link::Ready(_) => texts::available(),
        }
    }

    fn ready_ranges(&self) -> Result<Vec<f64>, KernelError> {
        match &self.link {
            Link::Ready(scope) => Ok(scope.ranges.clone()),
            Link::Down(_) => Err(KernelError::recoverable(texts::NEEDS_READY_SCOPE)),
        }
    }

    fn active(&mut self) -> Result<&mut Active, KernelError> {
        self.session
            .as_mut()
            .ok_or_else(|| KernelError::recoverable(NO_SESSION))
    }

    fn refresh(&mut self) {
        let unit = match &self.link {
            Link::Ready(Scope {
                unit_id: Some(id),
                ranges,
                ..
            }) => Some(unit_calibration(self.file.unit(id), ranges)),
            _ => None,
        };
        let session = self.session.as_ref().map(|a| a.machine.view());
        if let Some(state) = self.slice.update(self.availability(), unit, session) {
            self.env.bus.publish(ZeroCalibrationChanged(state));
        }
    }
}

fn wrong_step(_: crate::domain::session::WrongStep) -> KernelError {
    KernelError::recoverable(WRONG_STEP)
}
