use std::sync::Arc;
use std::time::{Duration, Instant};

use sweepscope_kernel::contracts::{
    Capabilities, CaptureCompleted, CaptureOutcome, ConnectionChanged, ConnectionState, DerivedTrace, DeviceIdentity,
    DeviceSource, Frame, FrameCaptured, RollPolled, TriggerOutcome,
};
use sweepscope_kernel::KernelError;

use super::autoset;
use super::derive::derive_trace;
use super::effects::{Call, Effects, Output};
use super::roll::RollSessions;
use crate::domain::autoset_run::{AutosetRun, Finish, Next, FAILED_TEXT, NO_SIGNAL_TEXT};
use crate::domain::setup_file::SetupFile;
use crate::{
    next_status, AcquisitionStatus, AutosetOutcomeKind, AutosetStatus, CaptureMeter, Change, ControlsAvailability,
    Defaults, FormattedValue, Instrument, InstrumentFacts, InstrumentSnapshot, Phase, SetupReport, StatusEvent,
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
    autoset: AutosetRun,
    roll: RollSessions,
    facts: InstrumentFacts,
    instrument_revision: u64,
    setup_revision: u64,
    session: Option<Session>,
    last_facts_at: Option<Instant>,
    saved_setup: Option<SetupFile>,
    unsaved_since: Option<Instant>,
    held: Option<HeldFrame>,
}

/// The newest published frame, kept so a math change while stopped can be shown on it.
struct HeldFrame {
    frame: Arc<Frame>,
    setup_revision: u64,
    derived: Option<Arc<DerivedTrace>>,
}

impl Core {
    pub fn new() -> Self {
        Self {
            instrument: Instrument::new(Defaults::default()),
            meter: CaptureMeter::default(),
            status: AcquisitionStatus {
                revision: 1,
                phase: Phase::Stopped,
                autoset: AutosetStatus::default(),
            },
            autoset: Default::default(),
            roll: RollSessions::default(),
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
            held: None,
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
        self.roll.sync(&self.instrument, &mut fx.outputs);
        self.track_setup(fx, now);
        self.rederive_held(fx);
    }

    /// While stopped the screen shows the held frame, so a changed math trace must be derived from it again.
    fn rederive_held(&mut self, fx: &mut Effects) {
        if self.instrument.running() {
            return;
        }
        let Some(held) = self
            .held
            .as_mut()
            .filter(|held| held.setup_revision == self.setup_revision)
        else {
            return;
        };
        let derived = derive_trace(&self.instrument, &held.frame);
        if derived == held.derived {
            return;
        }
        held.derived = derived.clone();
        fx.outputs.push(Output::Frame(FrameCaptured {
            frame: Arc::clone(&held.frame),
            context: self.instrument.frame_context(self.setup_revision),
            derived,
        }));
    }

    pub fn set_running(&mut self, on: bool, fx: &mut Effects, now: Instant) -> Result<(), KernelError> {
        let was_single = self.instrument.single_armed();
        let change = self.instrument.set_running(on)?;
        if was_single {
            self.send_setup(fx);
        }
        if change.state_changed {
            fx.calls.push(Call::Capturing(on));
            self.meter.clear();
            self.advance_phase(if on { StatusEvent::Run } else { StatusEvent::Stop }, fx);
            self.publish_instrument(fx);
            self.refresh_facts(fx, now, true);
        }
        self.roll.sync(&self.instrument, &mut fx.outputs);
        Ok(())
    }

    /// One capture that waits for a trigger; the device runs with `Normal` sweep until the frame comes.
    pub fn arm_single(&mut self, fx: &mut Effects, now: Instant) -> Result<(), KernelError> {
        self.require(|controls| &controls.single, "Single is not available")?;
        self.instrument.arm_single()?;
        self.send_setup(fx);
        fx.calls.push(Call::Capturing(true));
        self.meter.clear();
        self.advance_phase(StatusEvent::ArmSingle, fx);
        self.publish_instrument(fx);
        self.refresh_facts(fx, now, true);
        Ok(())
    }

    fn end_single(&mut self, fx: &mut Effects, now: Instant) {
        if self.instrument.finish_single().is_err() {
            return;
        }
        self.send_setup(fx);
        fx.calls.push(Call::Capturing(false));
        self.meter.clear();
        self.advance_phase(StatusEvent::SingleDone, fx);
        self.publish_instrument(fx);
        self.refresh_facts(fx, now, true);
        self.track_setup(fx, now);
    }

    pub fn force_trigger(&mut self, fx: &mut Effects) -> Result<(), KernelError> {
        self.require(|controls| &controls.force, "Force trigger is not available")?;
        fx.calls.push(Call::Force);
        Ok(())
    }

    /// Ends an armed Single, then probes until the machine decides; the request returns at once.
    pub fn start_autoset(&mut self, fx: &mut Effects, now: Instant) -> Result<(), KernelError> {
        self.require(|controls| &controls.autoset, "Auto-setup is not available")?;
        if self.instrument.single_armed() {
            self.set_running(false, fx, now)?;
        }
        let start = self
            .instrument
            .autoset_start()
            .ok_or_else(|| KernelError::recoverable(NO_SCOPE))?;
        let was_running = self.instrument.running();
        let next = self.autoset.begin(start, now, was_running);
        if self.instrument.set_autoset_running(true) {
            self.publish_instrument(fx);
        }
        self.sync_autoset(fx);
        if !was_running {
            fx.calls.push(Call::Capturing(true));
        }
        self.advance_autoset(next, fx, now);
        Ok(())
    }

    fn advance_autoset(&mut self, next: Next, fx: &mut Effects, now: Instant) {
        match next {
            Next::Probe(plan) => {
                self.setup_revision += 1;
                let sent = autoset::send_probe(&self.instrument, &mut self.autoset, self.setup_revision, &plan, fx);
                if !sent {
                    self.finish_autoset(Finish::Failed, fx, now);
                }
            }
            Next::Finish(finish) => self.finish_autoset(finish, fx, now),
        }
    }

    /// Done and no signal make the plan the user's setup; a failure sends the untouched setup back to the device.
    fn finish_autoset(&mut self, finish: Finish, fx: &mut Effects, now: Instant) {
        let was_running = self.autoset.was_running();
        self.instrument.set_autoset_running(false);
        let applied = match finish {
            Finish::Done(plan) => self.apply_autoset_plan(&plan, AutosetOutcomeKind::Done, fx, now),
            Finish::NoSignal(plan) => self.apply_autoset_plan(&plan, AutosetOutcomeKind::NoSignal, fx, now),
            Finish::Failed => false,
        };
        if !applied {
            self.restore_after_autoset(was_running, fx, now);
        }
        self.sync_autoset(fx);
    }

    fn apply_autoset_plan(
        &mut self,
        plan: &crate::domain::autoset_run::Plan,
        kind: AutosetOutcomeKind,
        fx: &mut Effects,
        now: Instant,
    ) -> bool {
        if self.instrument.autoset_apply(plan).is_err() {
            return false;
        }
        let text = match kind {
            AutosetOutcomeKind::NoSignal => NO_SIGNAL_TEXT.to_owned(),
            _ => self.instrument.autoset_summary().unwrap_or_default(),
        };
        self.autoset.conclude(kind, text);
        self.meter.clear();
        self.advance_phase(StatusEvent::Run, fx);
        let change = Change {
            setup_changed: true,
            state_changed: true,
        };
        self.after_change(change, fx, now);
        true
    }

    fn restore_after_autoset(&mut self, was_running: bool, fx: &mut Effects, now: Instant) {
        self.autoset
            .conclude(AutosetOutcomeKind::Failed, FAILED_TEXT.to_owned());
        self.send_setup(fx);
        if !was_running {
            fx.calls.push(Call::Capturing(false));
        }
        self.publish_instrument(fx);
        self.refresh_facts(fx, now, true);
    }

    /// The scope is gone: nothing to send, the setup was never changed, the outcome says so.
    fn abort_autoset(&mut self, fx: &mut Effects) {
        if !self.autoset.active() {
            return;
        }
        self.instrument.set_autoset_running(false);
        self.autoset
            .conclude(AutosetOutcomeKind::Failed, FAILED_TEXT.to_owned());
        self.sync_autoset(fx);
    }

    fn require(
        &self,
        control: fn(&ControlsAvailability) -> &crate::Availability,
        fallback: &str,
    ) -> Result<(), KernelError> {
        let state = self
            .instrument
            .state(self.instrument_revision)
            .ok_or_else(|| KernelError::recoverable(NO_SCOPE))?;
        let availability = control(&state.controls);
        if availability.enabled {
            return Ok(());
        }
        Err(KernelError::recoverable(
            availability.reason.clone().unwrap_or_else(|| fallback.to_owned()),
        ))
    }

    /// Publishes the auto-setup status when it changed.
    fn sync_autoset(&mut self, fx: &mut Effects) {
        let status = self.autoset.status();
        if status == self.status.autoset {
            return;
        }
        self.status = AcquisitionStatus {
            revision: self.status.revision + 1,
            phase: self.status.phase,
            autoset: status,
        };
        fx.outputs.push(Output::Status(self.status.clone()));
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
        self.roll.sync(&self.instrument, &mut fx.outputs);
        self.track_setup(fx, now);
    }

    pub fn capture(&mut self, completed: CaptureCompleted, fx: &mut Effects, now: Instant) {
        if self.autoset.active() {
            if let Some(next) = autoset::consume(&mut self.autoset, completed, now) {
                self.advance_autoset(next, fx, now);
            }
            return;
        }
        if !self.instrument.running() || completed.setup_revision != self.setup_revision {
            return;
        }
        match completed.outcome {
            CaptureOutcome::Frame(frame) => {
                let single = self.instrument.single_armed();
                if let Some(event) = status_event(frame.trigger).filter(|_| !single) {
                    self.advance_phase(event, fx);
                }
                self.meter.record(now);
                self.autoset.note_frame(&frame);
                self.sync_autoset(fx);
                let context = self.instrument.frame_context(self.setup_revision);
                let derived = derive_trace(&self.instrument, &frame);
                self.held = Some(HeldFrame {
                    frame: Arc::clone(&frame),
                    setup_revision: self.setup_revision,
                    derived: derived.clone(),
                });
                fx.outputs.push(Output::Frame(FrameCaptured {
                    frame,
                    context,
                    derived,
                }));
                if single {
                    self.end_single(fx, now);
                    return;
                }
                self.refresh_facts(fx, now, false);
                self.track_setup(fx, now);
            }
            CaptureOutcome::TimedOut => self.advance_phase(StatusEvent::TimedOut, fx),
        }
    }

    /// Roll items of the current setup only; a poll of an older revision belongs to a session that is over.
    pub fn roll_polled(&mut self, polled: RollPolled, fx: &mut Effects) {
        if polled.setup_revision != self.setup_revision {
            return;
        }
        let Some(captured) = self.roll.item(polled) else {
            return;
        };
        fx.outputs.push(Output::Roll(captured));
        self.advance_phase(StatusEvent::Auto, fx);
    }

    pub fn tick(&mut self, fx: &mut Effects, now: Instant) {
        if let Some(next) = autoset::expire(&self.autoset, now) {
            self.advance_autoset(next, fx, now);
        }
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
        self.abort_autoset(fx);
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
        let Some(setup) = self.instrument.device_setup() else {
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
                autoset: self.status.autoset.clone(),
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
