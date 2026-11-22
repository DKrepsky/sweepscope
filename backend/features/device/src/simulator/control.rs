//! Fault and signal injection for the simulator, shared between the device and the catalog.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};

use super::signal::SignalKind;
use crate::{CatalogEvent, DeviceId};

pub const DEVICE_ID: &str = "simulated-6254bc";

pub fn device_id() -> DeviceId {
    DeviceId(DEVICE_ID.to_owned())
}

struct State {
    plugged: bool,
    overrun_pending: bool,
    signals: [SignalKind; 4],
}

struct Shared {
    state: Mutex<State>,
    listeners: Mutex<Vec<Sender<CatalogEvent>>>,
}

/// Handle to the simulated instrument's faults. Clones control the same instrument.
#[derive(Clone)]
pub struct SimulatorControl {
    shared: Arc<Shared>,
}

impl Default for SimulatorControl {
    fn default() -> Self {
        Self::new()
    }
}

impl SimulatorControl {
    /// A plugged-in simulator with a sine on every channel.
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Shared {
                state: Mutex::new(State {
                    plugged: true,
                    overrun_pending: false,
                    signals: [SignalKind::Sine; 4],
                }),
                listeners: Mutex::new(Vec::new()),
            }),
        }
    }

    /// Every device call fails with `Disconnected` until `plug_in`; the catalog emits `Detached` once.
    pub fn unplug(&self) {
        if self.set_plugged(false) {
            self.emit(CatalogEvent::Detached(device_id()));
        }
    }

    pub fn plug_in(&self) {
        if self.set_plugged(true) {
            self.emit(CatalogEvent::Attached(device_id()));
        }
    }

    /// The next roll poll returns `Overrun` once.
    pub fn inject_overrun(&self) {
        self.state().overrun_pending = true;
    }

    /// Channels beyond the fourth are ignored.
    pub fn set_signal(&self, channel: usize, kind: SignalKind) {
        if let Some(slot) = self.state().signals.get_mut(channel) {
            *slot = kind;
        }
    }

    pub(super) fn is_plugged(&self) -> bool {
        self.state().plugged
    }

    pub(super) fn take_overrun(&self) -> bool {
        std::mem::take(&mut self.state().overrun_pending)
    }

    pub(super) fn signal(&self, channel: usize) -> SignalKind {
        self.state().signals.get(channel).copied().unwrap_or(SignalKind::Sine)
    }

    pub(super) fn subscribe(&self) -> Receiver<CatalogEvent> {
        let (sender, receiver) = channel();
        self.listeners().push(sender);
        receiver
    }

    /// Returns whether the state changed.
    fn set_plugged(&self, plugged: bool) -> bool {
        let mut state = self.state();
        let changed = state.plugged != plugged;
        state.plugged = plugged;
        changed
    }

    fn emit(&self, event: CatalogEvent) {
        self.listeners().retain(|listener| listener.send(event.clone()).is_ok());
    }

    fn state(&self) -> MutexGuard<'_, State> {
        self.shared
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn listeners(&self) -> MutexGuard<'_, Vec<Sender<CatalogEvent>>> {
        self.shared
            .listeners
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_a_plugged_simulator_when_unplugged_then_it_reports_unplugged_and_emits_detached_once() {
        let control = SimulatorControl::new();
        let events = control.subscribe();

        control.unplug();
        control.unplug();

        assert!(!control.is_plugged());
        assert_eq!(events.try_recv(), Ok(CatalogEvent::Detached(device_id())));
        assert!(events.try_recv().is_err());
    }

    #[test]
    fn given_an_unplugged_simulator_when_plugged_in_then_it_emits_attached() {
        let control = SimulatorControl::new();
        control.unplug();
        let events = control.subscribe();

        control.plug_in();

        assert!(control.is_plugged());
        assert_eq!(events.try_recv(), Ok(CatalogEvent::Attached(device_id())));
    }

    #[test]
    fn given_an_injected_overrun_when_taken_twice_then_only_the_first_sees_it() {
        let control = SimulatorControl::new();
        control.inject_overrun();

        assert!(control.take_overrun());
        assert!(!control.take_overrun());
    }

    #[test]
    fn given_a_signal_set_on_a_channel_when_read_then_other_channels_keep_their_signal() {
        let control = SimulatorControl::new();

        control.set_signal(2, SignalKind::Square);

        assert_eq!(control.signal(2), SignalKind::Square);
        assert_eq!(control.signal(0), SignalKind::Sine);
    }

    #[test]
    fn given_a_dropped_subscriber_when_an_event_is_emitted_then_the_others_still_receive_it() {
        let control = SimulatorControl::new();
        drop(control.subscribe());
        let alive = control.subscribe();

        control.unplug();

        assert_eq!(alive.try_recv(), Ok(CatalogEvent::Detached(device_id())));
    }
}
