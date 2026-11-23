#![allow(dead_code)]
//! Rig for the acquisition module tests. A feature crate may not depend on `device`, so the device side is a fake
//! that answers the three kernel requests and publishes the two events the real module publishes.
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use sweepscope_acquisition::{
    AcquisitionDeps, AcquisitionModule, FactsChanged, GetInstrument, InstrumentChanged, InstrumentSnapshot,
    StatusChanged,
};
use sweepscope_kernel::contracts::{
    AcquisitionSetup, ApplySetup, CaptureCompleted, CaptureOutcome, ChannelData, ConnectionChanged, ConnectionIssue,
    ConnectionIssueKind, ConnectionSnapshot, ConnectionState, DeviceIdentity, DeviceSource, ForceTrigger, Frame,
    FrameCaptured, SetCapturing, TriggerOutcome,
};
use sweepscope_kernel::testing::{eventually, EventRecorder, FakeClock, MemoryStorage, ScriptedPicker};
use sweepscope_kernel::{Bus, KernelError};

use super::domain_support::capabilities;

pub const WAIT: Duration = Duration::from_secs(3);

/// What the fake device was asked to do, in order.
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceCall {
    Apply(u64, Box<AcquisitionSetup>),
    Capturing(bool),
    Force,
}

#[derive(Default)]
pub struct FakeDevice {
    calls: Mutex<Vec<DeviceCall>>,
}

impl FakeDevice {
    fn attach(self: &Arc<Self>, bus: &Bus) {
        let device = Arc::clone(self);
        bus.handle(move |request: ApplySetup| {
            device.record(DeviceCall::Apply(request.setup_revision, Box::new(request.setup)));
            Ok(())
        })
        .unwrap();
        let device = Arc::clone(self);
        bus.handle(move |request: SetCapturing| {
            device.record(DeviceCall::Capturing(request.on));
            Ok(())
        })
        .unwrap();
        let device = Arc::clone(self);
        bus.handle(move |_: ForceTrigger| {
            device.record(DeviceCall::Force);
            Ok(())
        })
        .unwrap();
    }

    fn record(&self, call: DeviceCall) {
        self.calls.lock().unwrap().push(call);
    }

    pub fn calls(&self) -> Vec<DeviceCall> {
        self.calls.lock().unwrap().clone()
    }

    pub fn applies(&self) -> Vec<(u64, AcquisitionSetup)> {
        self.calls()
            .into_iter()
            .filter_map(|call| match call {
                DeviceCall::Apply(revision, setup) => Some((revision, *setup)),
                _ => None,
            })
            .collect()
    }

    pub fn capturing_calls(&self) -> Vec<bool> {
        self.calls()
            .into_iter()
            .filter_map(|call| match call {
                DeviceCall::Capturing(on) => Some(on),
                _ => None,
            })
            .collect()
    }

    pub fn forces(&self) -> usize {
        self.calls().iter().filter(|call| **call == DeviceCall::Force).count()
    }
}

pub struct Rig {
    pub bus: Bus,
    pub clock: FakeClock,
    pub device: Arc<FakeDevice>,
    pub instrument: EventRecorder<InstrumentChanged>,
    pub status: EventRecorder<StatusChanged>,
    pub facts: EventRecorder<FactsChanged>,
    pub frames: EventRecorder<FrameCaptured>,
    pub storage: MemoryStorage,
    pub module: AcquisitionModule,
    connection_revision: AtomicU64,
}

/// Recorders exist before the module so no first event is missed.
pub fn rig() -> Rig {
    rig_with(MemoryStorage::default(), ScriptedPicker::default())
}

/// A rig on given storage (a second module can reopen it) and a scripted picker.
pub fn rig_with(storage: MemoryStorage, picker: ScriptedPicker) -> Rig {
    let bus = Bus::new();
    let clock = FakeClock::default();
    let device = Arc::new(FakeDevice::default());
    device.attach(&bus);
    let instrument = EventRecorder::new(&bus);
    let status = EventRecorder::new(&bus);
    let facts = EventRecorder::new(&bus);
    let frames = EventRecorder::new(&bus);
    let module = AcquisitionModule::new(AcquisitionDeps {
        bus: bus.clone(),
        clock: Arc::new(clock.clone()),
        storage: Arc::new(storage.clone()),
        picker: Arc::new(picker),
    })
    .unwrap();
    Rig {
        bus,
        clock,
        device,
        instrument,
        status,
        facts,
        frames,
        storage,
        module,
        connection_revision: AtomicU64::new(1),
    }
}

pub fn ready_rig() -> Rig {
    let rig = rig();
    rig.plug();
    rig
}

pub fn identity() -> DeviceIdentity {
    DeviceIdentity {
        model: "Fake scope".into(),
        firmware: None,
        fpga_version: None,
        calibration: sweepscope_kernel::contracts::CalibrationStatus::Valid,
        simulated: true,
    }
}

impl Rig {
    pub fn snapshot(&self) -> InstrumentSnapshot {
        self.bus.request(GetInstrument).unwrap()
    }

    fn announce(&self, state: ConnectionState, issue: Option<ConnectionIssue>, ready: bool) {
        let ready_identity = ready.then(identity);
        self.bus.publish(ConnectionChanged {
            snapshot: ConnectionSnapshot {
                revision: self.connection_revision.fetch_add(1, Ordering::SeqCst) + 1,
                state,
                source: Some(DeviceSource::Simulator),
                identity: ready_identity,
                issue,
                simulator_active: true,
            },
            capabilities: ready.then(capabilities),
        });
    }

    /// Publishes `Ready` and waits until the module applied it.
    pub fn plug(&self) {
        let before = self.device.applies().len();
        self.announce(ConnectionState::Ready, None, true);
        self.wait(|| self.device.applies().len() > before);
    }

    pub fn unplug(&self) {
        self.announce(ConnectionState::Disconnected, None, false);
        self.wait_for_reason("No scope connected");
    }

    pub fn connecting(&self) {
        self.announce(ConnectionState::Connecting, None, false);
        self.wait_for_reason("Connecting");
    }

    pub fn fail(&self, message: &str) {
        let issue = ConnectionIssue {
            kind: ConnectionIssueKind::Communication,
            message: message.into(),
            help: None,
        };
        self.announce(ConnectionState::Error, Some(issue), false);
        self.wait_for_reason(message);
    }

    pub fn announce_ready_again(&self) {
        self.announce(ConnectionState::Ready, None, true);
    }

    fn wait_for_reason(&self, reason: &str) {
        self.wait(|| {
            self.snapshot()
                .instrument
                .is_some_and(|i| i.controls.run.reason.as_deref() == Some(reason))
        });
    }

    pub fn wait(&self, condition: impl FnMut() -> bool) {
        assert!(eventually(WAIT, condition), "condition did not hold within {WAIT:?}");
    }

    pub fn setup_revision(&self) -> u64 {
        self.device.applies().last().expect("a setup was applied").0
    }

    pub fn complete(&self, setup_revision: u64, outcome: CaptureOutcome) {
        self.bus.publish(CaptureCompleted {
            setup_revision,
            outcome,
        });
    }

    pub fn complete_triggered(&self, setup_revision: u64) {
        self.complete(
            setup_revision,
            CaptureOutcome::Frame(Arc::new(frame(TriggerOutcome::Triggered { sample_index: 2048 }))),
        );
    }

    pub fn complete_auto(&self, setup_revision: u64) {
        self.complete(
            setup_revision,
            CaptureOutcome::Frame(Arc::new(frame(TriggerOutcome::Auto))),
        );
    }

    /// The module handles events in order, so seeing the newest sentinel frame means every earlier event was handled.
    pub fn settle_after_capture_events(&self) {
        let seen = self.frames.events().len();
        self.complete_auto(self.setup_revision());
        self.wait(|| self.frames.events().len() > seen);
    }
}

pub fn frame(trigger: TriggerOutcome) -> Frame {
    Frame {
        channels: [
            Some(ChannelData {
                codes: vec![128; 4096],
                volts_per_div: 1.0,
                zero_code: 128.0,
            }),
            None,
            None,
            None,
        ],
        counts_per_div: 25.0,
        sample_interval_s: 1e-6,
        trigger,
    }
}

pub fn expect_recoverable(result: Result<(), KernelError>) -> String {
    let error = result.expect_err("the request must be rejected");
    assert_eq!(error.category, sweepscope_kernel::ErrorCategory::Recoverable);
    error.message
}
