//! Fault and signal injection for the simulator, shared between the device and the catalog.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};

use super::profile::{default_zero_error_volts, RANGE_COUNT};
use super::signal::{SignalKind, SignalSpec};
use crate::{CatalogEvent, DeviceId};

pub const DEVICE_ID: &str = "simulated-6254bc";

pub fn device_id() -> DeviceId {
    DeviceId(DEVICE_ID.to_owned())
}

struct State {
    plugged: bool,
    /// Counts unplugs; a device opened in an earlier generation stays disconnected.
    generation: u64,
    overrun_pending: bool,
    signals: [SignalSpec; 4],
    zero_error_volts: [[f64; RANGE_COUNT]; 4],
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
                    generation: 0,
                    overrun_pending: false,
                    signals: [SignalSpec::new(SignalKind::Sine); 4],
                    zero_error_volts: default_zero_error_volts(),
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
    pub fn set_signal(&self, channel: usize, signal: impl Into<SignalSpec>) {
        if let Some(slot) = self.state().signals.get_mut(channel) {
            *slot = signal.into();
        }
    }

    /// Zero error the simulated input adds to every signal on `range` (index into the vertical ranges).
    /// Out of range channels or ranges are ignored.
    pub fn set_zero_error_volts(&self, channel: usize, range: usize, volts: f64) {
        if let Some(slot) = self
            .state()
            .zero_error_volts
            .get_mut(channel)
            .and_then(|ranges| ranges.get_mut(range))
        {
            *slot = volts;
        }
    }

    pub(super) fn zero_error_volts(&self, channel: usize, range: usize) -> f64 {
        self.state()
            .zero_error_volts
            .get(channel)
            .and_then(|ranges| ranges.get(range))
            .copied()
            .unwrap_or(0.0)
    }

    pub(super) fn is_plugged(&self) -> bool {
        self.state().plugged
    }

    pub(super) fn generation(&self) -> u64 {
        self.state().generation
    }

    /// True while the instrument is plugged in and has not been re-enumerated since `generation`.
    pub(super) fn is_reachable(&self, generation: u64) -> bool {
        let state = self.state();
        state.plugged && state.generation == generation
    }

    pub(super) fn take_overrun(&self) -> bool {
        std::mem::take(&mut self.state().overrun_pending)
    }

    pub(super) fn signal(&self, channel: usize) -> SignalSpec {
        self.state()
            .signals
            .get(channel)
            .copied()
            .unwrap_or_else(|| SignalSpec::new(SignalKind::Sine))
    }

    pub(super) fn subscribe(&self) -> Receiver<CatalogEvent> {
        let (sender, receiver) = channel();
        if self.is_plugged() {
            let _ = sender.send(CatalogEvent::Attached(device_id()));
        }
        self.listeners().push(sender);
        receiver
    }

    /// Returns whether the state changed.
    fn set_plugged(&self, plugged: bool) -> bool {
        let mut state = self.state();
        let changed = state.plugged != plugged;
        state.plugged = plugged;
        if changed && !plugged {
            state.generation += 1;
        }
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
        events.try_recv().unwrap();

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
        assert!(events.try_recv().is_err());

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

        assert_eq!(control.signal(2), SignalSpec::new(SignalKind::Square));
        assert_eq!(control.signal(0), SignalSpec::new(SignalKind::Sine));
    }

    #[test]
    fn given_a_full_spec_set_on_a_channel_when_read_then_frequency_amplitude_and_offset_are_kept() {
        let control = SimulatorControl::new();
        let spec = SignalSpec::new(SignalKind::Triangle)
            .with_frequency(50.0)
            .with_amplitude(0.3)
            .with_offset(-0.2);

        control.set_signal(1, spec);

        assert_eq!(control.signal(1), spec);
    }

    #[test]
    fn given_a_zero_error_set_for_one_channel_and_range_when_read_then_others_keep_their_defaults() {
        let control = SimulatorControl::new();
        let other = control.zero_error_volts(1, 0);

        control.set_zero_error_volts(0, 3, 0.0123);

        assert_eq!(control.zero_error_volts(0, 3), 0.0123);
        assert_eq!(control.zero_error_volts(1, 0), other);
    }

    #[test]
    fn given_an_out_of_range_channel_or_range_when_set_then_it_is_ignored() {
        let control = SimulatorControl::new();

        control.set_zero_error_volts(4, 0, 1.0);
        control.set_zero_error_volts(0, 12, 1.0);

        assert_eq!(control.zero_error_volts(4, 0), 0.0);
        assert_eq!(control.zero_error_volts(0, 12), 0.0);
    }

    #[test]
    fn given_a_dropped_subscriber_when_an_event_is_emitted_then_the_others_still_receive_it() {
        let control = SimulatorControl::new();
        drop(control.subscribe());
        let alive = control.subscribe();
        alive.try_recv().unwrap();

        control.unplug();

        assert_eq!(alive.try_recv(), Ok(CatalogEvent::Detached(device_id())));
    }

    #[test]
    fn given_a_plugged_simulator_when_subscribed_then_attached_is_sent_first() {
        let control = SimulatorControl::new();

        let events = control.subscribe();

        assert_eq!(events.try_recv(), Ok(CatalogEvent::Attached(device_id())));
    }

    #[test]
    fn given_an_unplug_and_plug_in_when_reachability_is_asked_then_only_the_new_generation_is_reachable() {
        let control = SimulatorControl::new();
        let old = control.generation();

        control.unplug();
        control.plug_in();

        assert!(!control.is_reachable(old));
        assert!(control.is_reachable(control.generation()));
    }
}
