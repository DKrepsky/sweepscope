use std::sync::{Arc, Mutex, MutexGuard};

use sweepscope_kernel::contracts::{
    ConnectionChanged, ConnectionIssue, ConnectionSnapshot, ConnectionState, DeviceIdentity, DeviceSource,
};
use sweepscope_kernel::Bus;

use crate::Capabilities;

/// Everything of the connection slice except the revision and the simulator flag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub state: ConnectionState,
    pub source: Option<DeviceSource>,
    pub identity: Option<DeviceIdentity>,
    pub issue: Option<ConnectionIssue>,
}

impl Status {
    pub fn disconnected() -> Self {
        Self {
            state: ConnectionState::Disconnected,
            source: None,
            identity: None,
            issue: None,
        }
    }

    pub fn connecting(source: DeviceSource) -> Self {
        Self {
            state: ConnectionState::Connecting,
            source: Some(source),
            identity: None,
            issue: None,
        }
    }

    pub fn ready(source: DeviceSource, identity: DeviceIdentity) -> Self {
        Self {
            state: ConnectionState::Ready,
            source: Some(source),
            identity: Some(identity),
            issue: None,
        }
    }

    pub fn error(source: DeviceSource, issue: ConnectionIssue) -> Self {
        Self {
            state: ConnectionState::Error,
            source: Some(source),
            identity: None,
            issue: Some(issue),
        }
    }
}

struct Inner {
    snapshot: ConnectionSnapshot,
    capabilities: Option<Capabilities>,
    /// A deferred change waits for the next publish.
    unpublished: bool,
}

/// Single owner of the connection slice: assigns revisions and publishes `ConnectionChanged` only when something changed.
#[derive(Clone)]
pub struct ConnectionStore {
    inner: Arc<Mutex<Inner>>,
    bus: Bus,
}

impl ConnectionStore {
    /// Starts `Disconnected` at revision 1 and publishes it, so subscribers created before the module see the first state.
    pub fn new(bus: Bus) -> Self {
        let store = Self {
            inner: Arc::new(Mutex::new(Inner {
                snapshot: ConnectionSnapshot {
                    revision: 1,
                    state: ConnectionState::Disconnected,
                    source: None,
                    identity: None,
                    issue: None,
                    simulator_active: false,
                },
                capabilities: None,
                unpublished: false,
            })),
            bus,
        };
        store.bus.publish(store.event());
        store
    }

    pub fn snapshot(&self) -> ConnectionSnapshot {
        self.lock().snapshot.clone()
    }

    pub fn state(&self) -> ConnectionState {
        self.lock().snapshot.state
    }

    /// `capabilities` are kept only while the state is `Ready`.
    pub fn set_status(&self, status: Status, capabilities: Option<Capabilities>) {
        self.change(|inner| {
            let snapshot = &mut inner.snapshot;
            snapshot.state = status.state;
            snapshot.source = status.source;
            snapshot.identity = status.identity;
            snapshot.issue = status.issue;
            inner.capabilities = capabilities.filter(|_| status.state == ConnectionState::Ready);
        });
    }

    /// Records the flag without publishing, so a state change that follows carries it in one event; `flush` publishes
    /// it if no such change happened.
    pub fn defer_simulator_active(&self, active: bool) {
        let mut inner = self.lock();
        if inner.snapshot.simulator_active != active {
            inner.snapshot.simulator_active = active;
            inner.unpublished = true;
        }
    }

    pub fn flush(&self) {
        self.change(|_| {});
    }

    fn change(&self, apply: impl FnOnce(&mut Inner)) {
        let event = {
            let mut inner = self.lock();
            let before = (inner.snapshot.clone(), inner.capabilities.clone());
            apply(&mut inner);
            if !changed(&before, &inner) && !inner.unpublished {
                return;
            }
            inner.unpublished = false;
            inner.snapshot.revision += 1;
            ConnectionChanged {
                snapshot: inner.snapshot.clone(),
                capabilities: inner.capabilities.clone(),
            }
        };
        self.bus.publish(event);
    }

    fn event(&self) -> ConnectionChanged {
        let inner = self.lock();
        ConnectionChanged {
            snapshot: inner.snapshot.clone(),
            capabilities: inner.capabilities.clone(),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn changed(before: &(ConnectionSnapshot, Option<Capabilities>), after: &Inner) -> bool {
    before.0 != after.snapshot || before.1 != after.capabilities
}

#[cfg(test)]
mod tests {
    use sweepscope_kernel::contracts::CalibrationStatus;
    use sweepscope_kernel::testing::EventRecorder;

    use super::*;

    fn identity() -> DeviceIdentity {
        DeviceIdentity {
            model: "model".into(),
            firmware: None,
            fpga_version: None,
            calibration: CalibrationStatus::Valid,
            simulated: true,
            unit_id: None,
        }
    }

    fn capabilities() -> Capabilities {
        Capabilities {
            channel_count: 4,
            record_len: 4096,
            time_bases: vec![],
            vertical_ranges: vec![],
            adc_interval_ns: vec![],
            samples_per_div: 250.0,
        }
    }

    #[test]
    fn given_a_new_store_when_the_bus_has_a_subscriber_then_the_first_state_is_disconnected_at_revision_one() {
        let bus = Bus::new();
        let recorder = EventRecorder::<ConnectionChanged>::new(&bus);

        let store = ConnectionStore::new(bus);

        let snapshot = store.snapshot();
        assert_eq!(snapshot.revision, 1);
        assert_eq!(snapshot.state, ConnectionState::Disconnected);
        assert_eq!(recorder.events().len(), 1);
    }

    #[test]
    fn given_a_state_change_when_it_is_set_then_the_revision_grows_and_the_event_is_published() {
        let bus = Bus::new();
        let store = ConnectionStore::new(bus.clone());
        let recorder = EventRecorder::<ConnectionChanged>::new(&bus);

        store.set_status(Status::connecting(DeviceSource::Simulator), None);

        let events = recorder.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].snapshot.revision, 2);
        assert_eq!(events[0].snapshot.state, ConnectionState::Connecting);
    }

    #[test]
    fn given_the_same_state_when_it_is_set_again_then_nothing_is_published() {
        let bus = Bus::new();
        let store = ConnectionStore::new(bus.clone());
        store.set_status(Status::disconnected(), None);
        let recorder = EventRecorder::<ConnectionChanged>::new(&bus);

        store.set_status(Status::disconnected(), None);

        assert!(recorder.events().is_empty());
        assert_eq!(store.snapshot().revision, 1);
    }

    #[test]
    fn given_a_ready_device_when_the_simulator_flag_changes_then_the_event_still_carries_the_capabilities() {
        let bus = Bus::new();
        let store = ConnectionStore::new(bus.clone());
        store.set_status(Status::ready(DeviceSource::Hardware, identity()), Some(capabilities()));
        let recorder = EventRecorder::<ConnectionChanged>::new(&bus);

        store.defer_simulator_active(true);
        store.flush();

        let events = recorder.events();
        assert_eq!(events[0].capabilities, Some(capabilities()));
        assert!(events[0].snapshot.simulator_active);
    }

    #[test]
    fn given_a_deferred_flag_when_a_state_change_follows_then_one_event_carries_both() {
        let bus = Bus::new();
        let store = ConnectionStore::new(bus.clone());
        let recorder = EventRecorder::<ConnectionChanged>::new(&bus);

        store.defer_simulator_active(true);
        store.set_status(Status::connecting(DeviceSource::Simulator), None);
        store.flush();

        let events = recorder.events();
        assert_eq!(events.len(), 1);
        assert!(events[0].snapshot.simulator_active);
    }

    #[test]
    fn given_a_deferred_flag_when_no_state_change_follows_then_flush_publishes_it() {
        let bus = Bus::new();
        let store = ConnectionStore::new(bus.clone());
        let recorder = EventRecorder::<ConnectionChanged>::new(&bus);

        store.defer_simulator_active(true);
        store.flush();

        assert_eq!(recorder.events().len(), 1);
    }

    #[test]
    fn given_a_state_that_is_not_ready_when_capabilities_are_passed_then_they_are_dropped() {
        let bus = Bus::new();
        let store = ConnectionStore::new(bus.clone());
        let recorder = EventRecorder::<ConnectionChanged>::new(&bus);

        store.set_status(Status::connecting(DeviceSource::Hardware), Some(capabilities()));

        assert_eq!(recorder.events()[0].capabilities, None);
    }
}
