use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use sweepscope_kernel::contracts::{ApplySetup, CaptureCompleted, ConnectionChanged, ForceTrigger, SetCapturing};
use sweepscope_kernel::{Bus, Clock, KernelError};

use super::core::Core;
use super::effects::{Call, Effects, Output};
use crate::{FactsChanged, InstrumentChanged, InstrumentSnapshot, StatusChanged};

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The core behind two locks. `turn` orders whole steps (change, publish, device calls) so the device sees setups in
/// revision order; `core` is held only while the rules run, never across a bus call.
pub struct Shared {
    bus: Bus,
    clock: Arc<dyn Clock>,
    turn: Mutex<()>,
    core: Mutex<Core>,
}

impl Shared {
    pub fn new(bus: Bus, clock: Arc<dyn Clock>) -> Self {
        Self {
            bus,
            clock,
            turn: Mutex::new(()),
            core: Mutex::new(Core::new()),
        }
    }

    pub fn snapshot(&self) -> InstrumentSnapshot {
        lock(&self.core).snapshot()
    }

    /// One step of a request. The state change stays even when the device refuses a call; the error reaches the caller.
    pub fn act(
        &self,
        step: impl FnOnce(&mut Core, &mut Effects, Instant) -> Result<(), KernelError>,
    ) -> Result<(), KernelError> {
        let _turn = lock(&self.turn);
        let mut effects = Effects::default();
        let now = self.clock.now();
        let outcome = step(&mut lock(&self.core), &mut effects, now);
        let sent = self.dispatch(effects);
        outcome.and(sent)
    }

    /// Events from the bus. A device call that fails here means the device is already gone: its own
    /// `ConnectionChanged` follows, so there is nobody to tell.
    fn react(&self, step: impl FnOnce(&mut Core, &mut Effects, Instant)) {
        let _ = self.act(|core, effects, now| {
            step(core, effects, now);
            Ok(())
        });
    }

    pub fn connection(&self, change: ConnectionChanged) {
        self.react(|core, effects, now| core.connection(change, effects, now));
    }

    pub fn capture(&self, completed: CaptureCompleted) {
        self.react(|core, effects, now| core.capture(completed, effects, now));
    }

    pub fn tick(&self) {
        self.react(|core, effects, now| core.tick(effects, now));
    }

    fn dispatch(&self, effects: Effects) -> Result<(), KernelError> {
        for output in effects.outputs {
            self.publish(output);
        }
        let mut result = Ok(());
        for call in effects.calls {
            if let Err(error) = self.send(call) {
                result = result.and(Err(error));
            }
        }
        result
    }

    fn publish(&self, output: Output) {
        match output {
            Output::Instrument(state) => self.bus.publish(InstrumentChanged(*state)),
            Output::Status(status) => self.bus.publish(StatusChanged(status)),
            Output::Facts(facts) => self.bus.publish(FactsChanged(*facts)),
            Output::Frame(frame) => self.bus.publish(frame),
        }
    }

    fn send(&self, call: Call) -> Result<(), KernelError> {
        match call {
            Call::Apply(setup_revision, setup) => self.bus.request(ApplySetup { setup_revision, setup }),
            Call::Capturing(on) => self.bus.request(SetCapturing { on }),
            Call::Force => self.bus.request(ForceTrigger),
        }
    }
}
