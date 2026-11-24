use std::time::{Duration, Instant};

use sweepscope_kernel::contracts::{
    Capabilities, CaptureCompleted, CaptureOutcome, ConnectionChanged, ConnectionState, DeviceIdentity, DeviceSource,
    FrameCaptured, TriggerOutcome,
};
use sweepscope_kernel::KernelError;

use super::effects::{Call, Effects, Output};
use crate::domain::setup_file::SetupFile;
use crate::{
    next_status, AcquisitionStatus, CaptureMeter, Change, Defaults, FormattedValue, Instrument, InstrumentFacts,
    InstrumentSnapshot, Phase, SetupReport, StatusEvent,
};

const FACTS_EVERY: Duration = Duration::from_millis(250);
/// A change must stay this long before it is written, so a dragged control causes one write.
const AUTOSAVE_AFTER: Duration = Duration::from_secs(1);
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
    saved_setup: Option<SetupFile>,
    unsaved_since: Option<Instant>,
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
            saved_setup: None,
            unsaved_since: None,
        }
    }

    pub fn hold_last_setup(&mut self, file: SetupFile) {
        self.instrument.hold_setup(file);
    }

    pub fn note_setup(&mut self, text: String) {
        self.instrument.set_notice(text);
    }

    /// The bytes `SaveSetup` writes; fails before the first device was ready.
    pub fn setup_bytes(&self) -> Result<Vec<u8>, KernelError> {
        let file = self.instrument.to_setup_file();
        file.map(|f| f.to_bytes())
            .ok_or_else(|| KernelError::recoverable(NO_SCOPE))
    }

    /// Loading needs a ready device; the reason is the one the controls show.
    pub fn ensure_files_available(&self) -> Result<(), KernelError> {
        let state = self
            .instrument
            .state(0)
            .ok_or_else(|| KernelError::recoverable(NO_SCOPE))?;
        let files = state.controls.files;
        if files.enabled {
            return Ok(());
        }
        Err(KernelError::recoverable(
            files.reason.unwrap_or_else(|| NO_SCOPE.into()),
        ))
    }

    pub fn load_setup(&mut self, file: &SetupFile, fx: &mut Effects, now: Instant) -> Result<SetupReport, KernelError> {
        let before = (self.instrument.setup().cloned(), self.instrument.state(0));
        let report = self.instrument.apply_setup_file(file)?;
        let change = Change {
            setup_changed: self.instrument.setup().cloned() != before.0,
            state_changed: self.instrument.state(0) != before.1,
        };
        self.after_change(change, fx, now);
        Ok(report)
    }

    /// The setup that still has to reach `last-setup.json`, for the final flush.
    pub fn unsaved_setup(&self) -> Option<SetupFile> {
        self.instrument
            .to_setup_file()
            .filter(|current| self.saved_setup.as_ref() != Some(current))
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
        self.after_change(change, fx, now);
        Ok(())
    }

    fn after_change(&mut self, change: Change, fx: &mut Effects, now: Instant) {
        if change.setup_changed {
            self.send_setup(fx);
        }
        if change.state_changed {
            self.publish_instrument(fx);
            self.refresh_facts(fx, now, true);
        }
        self.track_setup(fx, now);
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
        self.track_setup(fx, now);
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
                self.track_setup(fx, now);
            }
            CaptureOutcome::TimedOut => self.advance_phase(StatusEvent::TimedOut, fx),
        }
    }

    pub fn tick(&mut self, fx: &mut Effects, now: Instant) {
        self.refresh_facts(fx, now, false);
        self.track_setup(fx, now);
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

    fn track_setup(&mut self, fx: &mut Effects, now: Instant) {
        let Some(current) = self.instrument.to_setup_file() else {
            return;
        };
        if self.saved_setup.as_ref() == Some(&current) {
            self.unsaved_since = None;
            return;
        }
        let since = *self.unsaved_since.get_or_insert(now);
        if now.saturating_duration_since(since) >= AUTOSAVE_AFTER {
            self.saved_setup = Some(current.clone());
            self.unsaved_since = None;
            fx.autosave = Some(current);
        }
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
