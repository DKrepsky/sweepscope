use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use serde_json::Value;
use sweepscope_acquisition::{FactsChanged, InstrumentChanged, StatusChanged};
use sweepscope_display::DisplayChanged;
use sweepscope_kernel::contracts::ConnectionChanged;
use sweepscope_kernel::{Bus, Event, KernelError, Subscription};
use sweepscope_measurement::MeasurementsChanged;

pub const CONNECTION_CHANGED: &str = "connection_changed";
pub const INSTRUMENT_CHANGED: &str = "instrument_changed";
pub const STATUS_CHANGED: &str = "status_changed";
pub const FACTS_CHANGED: &str = "facts_changed";
pub const DISPLAY_CHANGED: &str = "display_changed";
pub const MEASUREMENTS_CHANGED: &str = "measurements_changed";

const POLL: Duration = Duration::from_millis(50);

/// Where slice events go; the Tauri app handle in production, a recorder in tests.
pub trait EventEmitter: Send + Sync + 'static {
    fn emit(&self, name: &str, payload: Value);
}

impl EventEmitter for tauri::AppHandle {
    fn emit(&self, name: &str, payload: Value) {
        if let Err(error) = tauri::Emitter::emit(self, name, payload) {
            log::warn!("cannot emit {name}: {error}");
        }
    }
}

/// One thread per slice event, a plain forwarder with no logic of its own (ADR 0013 item 2).
/// Dropping it stops and joins the threads.
pub struct EventForwarders {
    stop: Arc<AtomicBool>,
    threads: Vec<JoinHandle<()>>,
}

impl EventForwarders {
    pub fn start(bus: &Bus, emitter: &Arc<dyn EventEmitter>) -> Result<Self, KernelError> {
        let mut forwarders = Self {
            stop: Arc::new(AtomicBool::new(false)),
            threads: Vec::new(),
        };
        forwarders.forward(bus, emitter, CONNECTION_CHANGED, |e: ConnectionChanged| {
            serde_json::to_value(e.snapshot)
        })?;
        forwarders.forward(bus, emitter, INSTRUMENT_CHANGED, |e: InstrumentChanged| {
            serde_json::to_value(e.0)
        })?;
        forwarders.forward(bus, emitter, STATUS_CHANGED, |e: StatusChanged| {
            serde_json::to_value(e.0)
        })?;
        forwarders.forward(bus, emitter, FACTS_CHANGED, |e: FactsChanged| serde_json::to_value(e.0))?;
        forwarders.forward(bus, emitter, DISPLAY_CHANGED, |e: DisplayChanged| {
            serde_json::to_value(e.0)
        })?;
        forwarders.forward(bus, emitter, MEASUREMENTS_CHANGED, |e: MeasurementsChanged| {
            serde_json::to_value(e.0)
        })?;
        Ok(forwarders)
    }

    fn forward<E: Event>(
        &mut self,
        bus: &Bus,
        emitter: &Arc<dyn EventEmitter>,
        name: &'static str,
        payload: fn(E) -> serde_json::Result<Value>,
    ) -> Result<(), KernelError> {
        let events = bus.subscribe::<E>();
        let emitter = Arc::clone(emitter);
        let stop = Arc::clone(&self.stop);
        let thread = std::thread::Builder::new()
            .name(format!("forward-{name}"))
            .spawn(move || run(&events, &stop, emitter.as_ref(), name, payload))
            .map_err(|error| KernelError::fatal(format!("cannot start the {name} forwarder: {error}")))?;
        self.threads.push(thread);
        Ok(())
    }
}

impl Drop for EventForwarders {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        for thread in self.threads.drain(..) {
            let _ = thread.join();
        }
    }
}

fn run<E>(
    events: &Subscription<E>,
    stop: &AtomicBool,
    emitter: &dyn EventEmitter,
    name: &str,
    payload: fn(E) -> serde_json::Result<Value>,
) {
    while !stop.load(Ordering::Acquire) {
        let Some(event) = events.recv_timeout(POLL) else {
            continue;
        };
        match payload(event) {
            Ok(value) => emitter.emit(name, value),
            Err(error) => log::error!("cannot serialise {name}: {error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use sweepscope_acquisition::{AcquisitionStatus, InstrumentFacts, Phase};
    use sweepscope_display::{DisplayState, InterpolationMode};
    use sweepscope_kernel::contracts::{ConnectionSnapshot, ConnectionState};
    use sweepscope_kernel::testing::eventually;
    use sweepscope_measurement::MeasurementsState;

    use super::*;

    const WAIT: Duration = Duration::from_secs(5);

    #[derive(Default)]
    struct Recorder {
        emitted: Mutex<Vec<(String, Value)>>,
    }

    impl Recorder {
        fn names(&self) -> Vec<String> {
            self.emitted
                .lock()
                .unwrap()
                .iter()
                .map(|(name, _)| name.clone())
                .collect()
        }

        fn payload(&self, name: &str) -> Option<Value> {
            let emitted = self.emitted.lock().unwrap();
            emitted.iter().find(|(n, _)| n == name).map(|(_, v)| v.clone())
        }
    }

    impl EventEmitter for Recorder {
        fn emit(&self, name: &str, payload: Value) {
            self.emitted.lock().unwrap().push((name.to_owned(), payload));
        }
    }

    fn started() -> (Bus, Arc<Recorder>, EventForwarders) {
        let bus = Bus::new();
        let recorder = Arc::new(Recorder::default());
        let emitter: Arc<dyn EventEmitter> = recorder.clone();
        let forwarders = EventForwarders::start(&bus, &emitter).expect("start");
        (bus, recorder, forwarders)
    }

    fn connection(revision: u64) -> ConnectionSnapshot {
        ConnectionSnapshot {
            revision,
            state: ConnectionState::Disconnected,
            source: None,
            identity: None,
            issue: None,
            simulator_active: false,
        }
    }

    fn facts(revision: u64) -> InstrumentFacts {
        InstrumentFacts {
            revision,
            sample_rate: None,
            sample_interval: None,
            record_points: None,
            capture_rate: None,
            dead_time: None,
        }
    }

    #[test]
    fn given_a_connection_change_when_published_then_the_snapshot_is_emitted_as_connection_changed() {
        let (bus, recorder, _forwarders) = started();

        bus.publish(ConnectionChanged {
            snapshot: connection(7),
            capabilities: None,
        });

        assert!(eventually(WAIT, || recorder.payload(CONNECTION_CHANGED).is_some()));
        let payload = recorder.payload(CONNECTION_CHANGED).expect("payload");
        assert_eq!(payload["revision"], 7);
        assert_eq!(payload["state"], "disconnected");
    }

    #[test]
    fn given_status_and_facts_changes_when_published_then_each_is_emitted_under_its_own_name() {
        let (bus, recorder, _forwarders) = started();

        bus.publish(StatusChanged(AcquisitionStatus {
            revision: 3,
            phase: Phase::Armed,
        }));
        bus.publish(FactsChanged(facts(4)));

        assert!(eventually(WAIT, || recorder.names().len() == 2));
        assert_eq!(recorder.payload(STATUS_CHANGED).expect("status")["revision"], 3);
        assert_eq!(recorder.payload(FACTS_CHANGED).expect("facts")["revision"], 4);
    }

    #[test]
    fn given_display_and_measurement_changes_when_published_then_each_is_emitted_under_its_own_name() {
        let (bus, recorder, _forwarders) = started();

        bus.publish(DisplayChanged(DisplayState {
            revision: 5,
            interpolation: InterpolationMode::Sinc,
            automatic: true,
        }));
        bus.publish(MeasurementsChanged(MeasurementsState {
            revision: 6,
            channels: Vec::new(),
            accuracy_note: String::new(),
        }));

        assert!(eventually(WAIT, || recorder.names().len() == 2));
        let display = recorder.payload(DISPLAY_CHANGED).expect("display");
        assert_eq!(display["revision"], 5);
        assert_eq!(display["interpolation"], "sinc");
        assert_eq!(
            recorder.payload(MEASUREMENTS_CHANGED).expect("measurements")["revision"],
            6
        );
    }

    #[test]
    fn given_the_forwarders_are_dropped_when_an_event_is_published_then_nothing_is_emitted() {
        let (bus, recorder, forwarders) = started();
        drop(forwarders);

        bus.publish(ConnectionChanged {
            snapshot: connection(1),
            capabilities: None,
        });

        assert!(recorder.names().is_empty());
    }
}
