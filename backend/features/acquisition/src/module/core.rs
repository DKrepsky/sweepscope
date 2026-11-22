use std::time::{Duration, Instant};

use sweepscope_kernel::contracts::{
    Capabilities, CaptureCompleted, CaptureOutcome, ConnectionChanged, ConnectionState, DeviceIdentity, DeviceSource,
    FrameCaptured, TriggerOutcome,
};
use sweepscope_kernel::KernelError;

use super::effects::{Call, Effects, Output};
use crate::{
    next_status, AcquisitionStatus, CaptureMeter, Change, Defaults, FormattedValue, Instrument, InstrumentFacts,
    InstrumentSnapshot, Phase, StatusEvent,
};

const FACTS_EVERY: Duration = Duration::from_millis(250);
const NO_SCOPE: &str = "No scope connected";
const CONNECTING: &str = "Connecting";
const UNKNOWN_ERROR: &str = "The scope reported an error";

type Session = (Option<DeviceSource>, Option<DeviceIdentity>);

/// All mutable state of the module and every rule that changes it. It never touches the bus: it returns `Effects`.
pub struct Core {
    instrument: Instrument,
    meter: CaptureMeter,
    status: AcquisitionStatus,
    facts: InstrumentFacts,
    instrument_revision: u64,
    setup_revision: u64,
    session: Option<Session>,
    last_facts_at: Option<Instant>,
}

impl Core {
    pub fn new() -> Self {
        Self {
            instrument: Instrument::new(Defaults::default()),
            meter: CaptureMeter::default(),
            status: AcquisitionStatus {
                revision: 1,
                phase: Phase::Stopped,
            },
            facts: InstrumentFacts {
                revision: 1,
                sample_rate: None,
                sample_interval: None,
                record_points: None,
                capture_rate: None,
                dead_time: None,
            },
            instrument_revision: 0,
            setup_revision: 0,
            session: None,
            last_facts_at: None,
        }
    }

    pub fn snapshot(&self) -> InstrumentSnapshot {
        InstrumentSnapshot {
            instrument: self.instrument.state(self.instrument_revision),
            status: self.status.clone(),
            facts: self.facts.clone(),
        }
    }

    pub fn edit(
        &mut self,
        fx: &mut Effects,
        now: Instant,
        change: impl FnOnce(&mut Instrument) -> Result<Change, KernelError>,
    ) -> Result<(), KernelError> {
        let change = change(&mut self.instrument)?;
        if change.setup_changed {
            self.send_setup(fx);
        }
        if change.state_changed {
            self.publish_instrument(fx);
            self.refresh_facts(fx, now, true);
        }
        Ok(())
    }

    pub fn set_running(&mut self, on: bool, fx: &mut Effects, now: Instant) -> Result<(), KernelError> {
        let change = self.instrument.set_running(on)?;
        if change.state_changed {
            fx.calls.push(Call::Capturing(on));
            self.meter.clear();
            self.advance_phase(if on { StatusEvent::Run } else { StatusEvent::Stop }, fx);
            self.publish_instrument(fx);
            self.refresh_facts(fx, now, true);
        }
        Ok(())
    }

    pub fn force_trigger(&mut self, fx: &mut Effects) -> Result<(), KernelError> {
        let state = self
            .instrument
            .state(self.instrument_revision)
            .ok_or_else(|| KernelError::recoverable(NO_SCOPE))?;
        if !state.controls.force.enabled {
            let reason = state.controls.force.reason;
            return Err(KernelError::recoverable(
                reason.unwrap_or_else(|| "Force trigger is not available".into()),
            ));
        }
        fx.calls.push(Call::Force);
        Ok(())
    }

    pub fn connection(&mut self, change: ConnectionChanged, fx: &mut Effects, now: Instant) {
        let before = self.instrument.state(0);
        let ConnectionChanged { snapshot, capabilities } = change;
        match (snapshot.state, capabilities) {
            (ConnectionState::Ready, Some(caps)) => self.become_ready((snapshot.source, snapshot.identity), caps, fx),
            (ConnectionState::Ready, None) => {}
            (ConnectionState::Connecting, _) => self.lose(Some(CONNECTING), fx),
            (ConnectionState::Error, _) => {
                let reason = snapshot
                    .issue
                    .map_or_else(|| UNKNOWN_ERROR.to_owned(), |issue| issue.message);
                self.lose(Some(&reason), fx);
            }
            (ConnectionState::Disconnected, _) => self.lose(None, fx),
        }
        if self.instrument.state(0) != before {
            self.publish_instrument(fx);
            self.refresh_facts(fx, now, true);
        }
    }

    pub fn capture(&mut self, completed: CaptureCompleted, fx: &mut Effects, now: Instant) {
        if !self.instrument.running() || completed.setup_revision != self.setup_revision {
            return;
        }
        match completed.outcome {
            CaptureOutcome::Frame(frame) => {
                if let Some(event) = status_event(frame.trigger) {
                    self.advance_phase(event, fx);
                }
                self.meter.record(now);
                let context = self.instrument.frame_context(self.setup_revision);
                fx.outputs.push(Output::Frame(FrameCaptured { frame, context }));
                self.refresh_facts(fx, now, false);
            }
            CaptureOutcome::TimedOut => self.advance_phase(StatusEvent::TimedOut, fx),
        }
    }

    pub fn tick(&mut self, fx: &mut Effects, now: Instant) {
        self.refresh_facts(fx, now, false);
    }

    /// A repeated `Ready` of the same session (for example a changed simulator flag) restores nothing.
    fn become_ready(&mut self, session: Session, caps: Capabilities, fx: &mut Effects) {
        if self.session.as_ref() == Some(&session) {
            return;
        }
        self.session = Some(session);
        let restore = self.instrument.on_ready(caps);
        self.send_setup(fx);
        if restore.resume {
            fx.calls.push(Call::Capturing(true));
            self.meter.clear();
            self.advance_phase(StatusEvent::Run, fx);
        }
    }

    /// `reason` is the text of an in-between state; `None` means the scope is gone.
    fn lose(&mut self, reason: Option<&str>, fx: &mut Effects) {
        match reason {
            Some(reason) => self.instrument.set_unavailable(reason),
            None => self.instrument.on_lost(),
        }
        self.session = None;
        self.meter.clear();
        self.advance_phase(StatusEvent::Lost, fx);
    }

    fn send_setup(&mut self, fx: &mut Effects) {
        let Some(setup) = self.instrument.setup().cloned() else {
            return;
        };
        self.setup_revision += 1;
        fx.calls.push(Call::Apply(self.setup_revision, setup));
    }

    fn publish_instrument(&mut self, fx: &mut Effects) {
        let next = self.instrument_revision + 1;
        if let Some(state) = self.instrument.state(next) {
            self.instrument_revision = next;
            fx.outputs.push(Output::Instrument(Box::new(state)));
        }
    }

    fn advance_phase(&mut self, event: StatusEvent, fx: &mut Effects) {
        let phase = next_status(self.status.phase, event);
        if phase != self.status.phase {
            self.status = AcquisitionStatus {
                revision: self.status.revision + 1,
                phase,
            };
            fx.outputs.push(Output::Status(self.status.clone()));
        }
    }

    /// Frame-driven refreshes are held back to one per 250 ms; setup and run changes publish at once.
    fn refresh_facts(&mut self, fx: &mut Effects, now: Instant, forced: bool) {
        let held_back = self
            .last_facts_at
            .is_some_and(|at| now.saturating_duration_since(at) < FACTS_EVERY);
        if held_back && !forced {
            return;
        }
        let Some(next) = self.instrument.facts(self.facts.revision + 1, &self.meter, now) else {
            return;
        };
        if texts(&next) != texts(&self.facts) {
            self.facts = next.clone();
            self.last_facts_at = Some(now);
            fx.outputs.push(Output::Facts(Box::new(next)));
        }
    }
}

fn status_event(outcome: TriggerOutcome) -> Option<StatusEvent> {
    match outcome {
        TriggerOutcome::Triggered { .. } => Some(StatusEvent::Triggered),
        TriggerOutcome::Auto => Some(StatusEvent::Auto),
        TriggerOutcome::None => None,
    }
}

type Shown<'a> = Option<(&'a str, &'a str)>;

fn shown(value: &Option<FormattedValue>) -> Shown<'_> {
    value.as_ref().map(|v| (v.text.as_str(), v.unit.as_str()))
}

fn texts(facts: &InstrumentFacts) -> [Shown<'_>; 5] {
    [
        shown(&facts.sample_rate),
        shown(&facts.sample_interval),
        shown(&facts.record_points),
        shown(&facts.capture_rate),
        shown(&facts.dead_time),
    ]
}
