#![allow(dead_code)]
//! A fake acquisition (hold, probe, answers) and a scripted connection around the real calibration module.
use std::sync::{Arc, Mutex};
use std::time::Duration;

use sweepscope_calibration::{
    CalibrationDeps, CalibrationModule, CloseZeroCalibration, ContinueZeroChannel, GetZeroCalibration,
    MeasureZeroChannel, OpenZeroCalibration, StartZeroCalibration, ZeroCalibrationChanged, ZeroCalibrationState,
    ZeroStep,
};
use sweepscope_kernel::contracts::{
    CalibrationStatus, Capabilities, CaptureProbe, ChannelData, ConnectionChanged, ConnectionSnapshot, ConnectionState,
    DeviceIdentity, DeviceSource, Frame, HoldInstrument, ProbeCaptured, TimeBaseStep, TriggerOutcome, VerticalRange,
    ZeroOffsetsChanged,
};
use sweepscope_kernel::testing::{EventRecorder, FakeClock, MemoryStorage};
use sweepscope_kernel::{Bus, KernelError, Storage, Subscription};

pub const UNIT: &str = "6254BC/pcb/0001";
pub const RANGES: [f64; 12] = [0.002, 0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0];
pub const COUNTS: f64 = 24.15;
pub const ZERO_CODE: f64 = 128.0;
/// The most a test waits for the module thread; it is a guard against hangs, never a measured bound.
const GUARD: Duration = Duration::from_secs(20);

pub fn capabilities() -> Capabilities {
    let step = |seconds_per_div: f64| TimeBaseStep {
        seconds_per_div,
        streaming: false,
        verified: true,
    };
    Capabilities {
        channel_count: 4,
        record_len: 4096,
        time_bases: vec![step(1e-6), step(1e-3), step(2e-3), step(5e-3)],
        vertical_ranges: RANGES
            .iter()
            .map(|&volts_per_div| VerticalRange {
                volts_per_div,
                verified: true,
                max_position_divs: 4.0,
            })
            .collect(),
        adc_interval_ns: vec![1.0, 2.0, 4.0, 4.0],
        samples_per_div: 250.0,
    }
}

pub fn connection(state: ConnectionState, unit_id: Option<&str>) -> ConnectionChanged {
    let ready = state == ConnectionState::Ready;
    ConnectionChanged {
        snapshot: ConnectionSnapshot {
            revision: 1,
            state,
            source: ready.then_some(DeviceSource::Hardware),
            identity: ready.then(|| DeviceIdentity {
                model: "6254BC".into(),
                firmware: None,
                fpga_version: None,
                calibration: CalibrationStatus::Valid,
                simulated: false,
                unit_id: unit_id.map(str::to_owned),
            }),
            issue: None,
            simulator_active: false,
        },
        capabilities: ready.then(capabilities),
    }
}

/// What the scope under the fake acquisition shows: codes of the channel under test at each range index.
pub type Signal = fn(channel: usize, range_index: usize) -> Vec<u8>;

/// A short with a positive zero error of 8 codes at 2 mV/div and 1 code elsewhere.
pub fn shorted(_channel: usize, range_index: usize) -> Vec<u8> {
    let bias = if range_index == 0 { 8 } else { 1 };
    vec![128 + bias; 300]
}

/// A 40-code square wave: far beyond what a short can show.
pub fn open_input(_channel: usize, _range_index: usize) -> Vec<u8> {
    (0..300).map(|i| if i % 2 == 0 { 148 } else { 108 }).collect()
}

pub struct Behaviour {
    pub signal: Signal,
    pub answers: bool,
    pub refuse_hold: Option<String>,
}

pub struct Acquisition {
    pub behaviour: Mutex<Behaviour>,
    pub holds: Mutex<Vec<bool>>,
    pub probes: Mutex<Vec<CaptureProbe>>,
}

impl Acquisition {
    pub fn hold_requests(&self) -> Vec<bool> {
        self.holds.lock().unwrap().clone()
    }

    pub fn probe_requests(&self) -> Vec<CaptureProbe> {
        self.probes.lock().unwrap().clone()
    }

    fn frames_for(&self, probe: &CaptureProbe) -> Vec<Arc<Frame>> {
        let channel = usize::from(probe.setup.trigger.source);
        let range_index = usize::from(probe.setup.channels[channel].range_index);
        let codes = (self.behaviour.lock().unwrap().signal)(channel, range_index);
        let data = || ChannelData {
            codes: codes.clone(),
            volts_per_div: RANGES[range_index],
            zero_code: ZERO_CODE,
        };
        (0..probe.keep)
            .map(|_| {
                Arc::new(Frame {
                    channels: [Some(data()), Some(data()), Some(data()), Some(data())],
                    counts_per_div: COUNTS,
                    sample_interval_s: 4e-8,
                    trigger: TriggerOutcome::Auto,
                })
            })
            .collect()
    }
}

pub struct Rig {
    pub bus: Bus,
    pub clock: FakeClock,
    pub storage: MemoryStorage,
    pub acquisition: Arc<Acquisition>,
    pub offsets: EventRecorder<ZeroOffsetsChanged>,
    changes: Subscription<ZeroCalibrationChanged>,
    _module: CalibrationModule,
}

pub fn fixed_time() -> f64 {
    1_700_000_000_000_000.0
}

/// A storage whose writes fail; reads see what was put in before.
pub struct ReadOnlyStorage(pub MemoryStorage);

impl Storage for ReadOnlyStorage {
    fn read(&self, key: &str) -> Result<Option<Vec<u8>>, KernelError> {
        self.0.read(key)
    }

    fn write(&self, _key: &str, _bytes: &[u8]) -> Result<(), KernelError> {
        Err(KernelError::recoverable("storage failed to write: PermissionDenied"))
    }
}

impl Rig {
    pub fn new() -> Self {
        let storage = MemoryStorage::default();
        Self::build(storage.clone(), Arc::new(storage))
    }

    /// `seen` is what the test inspects; `used` is what the module gets.
    pub fn build(seen: MemoryStorage, used: Arc<dyn Storage>) -> Self {
        let bus = Bus::new();
        let clock = FakeClock::default();
        let acquisition = Arc::new(Acquisition {
            behaviour: Mutex::new(Behaviour {
                signal: shorted,
                answers: true,
                refuse_hold: None,
            }),
            holds: Mutex::default(),
            probes: Mutex::default(),
        });
        install(&bus, &acquisition);
        let offsets = EventRecorder::new(&bus);
        let changes = bus.subscribe();
        let module = CalibrationModule::new(CalibrationDeps {
            bus: bus.clone(),
            clock: Arc::new(clock.clone()),
            storage: used,
            unix_time_us: fixed_time,
        })
        .unwrap();
        Self {
            bus,
            clock,
            storage: seen,
            acquisition,
            offsets,
            changes,
            _module: module,
        }
    }

    pub fn connect(&self, unit_id: Option<&str>) {
        self.bus.publish(connection(ConnectionState::Ready, unit_id));
        self.until(|state| state.availability.enabled || state.availability.reason.as_deref() != Some("No device"));
    }

    pub fn disconnect(&self) {
        self.bus.publish(connection(ConnectionState::Disconnected, None));
    }

    pub fn set_signal(&self, signal: Signal) {
        self.acquisition.behaviour.lock().unwrap().signal = signal;
    }

    pub fn state(&self) -> ZeroCalibrationState {
        self.bus.request(GetZeroCalibration).unwrap()
    }

    pub fn step(&self) -> Option<ZeroStep> {
        self.state().session.map(|s| s.step)
    }

    /// Waits until the slice satisfies `condition`; wakes on every published change.
    pub fn until(&self, condition: impl Fn(&ZeroCalibrationState) -> bool) -> ZeroCalibrationState {
        loop {
            let state = self.state();
            if condition(&state) {
                return state;
            }
            self.changes
                .recv_timeout(GUARD)
                .unwrap_or_else(|| panic!("the slice never reached the expected state, last: {state:?}"));
        }
    }

    pub fn until_step(&self, step: ZeroStep) -> ZeroCalibrationState {
        self.until(|state| state.session.as_ref().is_some_and(|s| s.step == step))
    }

    pub fn open(&self) -> Result<(), KernelError> {
        self.bus.request(OpenZeroCalibration)
    }

    pub fn start(&self) -> Result<(), KernelError> {
        self.bus.request(StartZeroCalibration)
    }

    pub fn measure(&self) -> Result<(), KernelError> {
        self.bus.request(MeasureZeroChannel)
    }

    pub fn continue_channel(&self) -> Result<(), KernelError> {
        self.bus.request(ContinueZeroChannel)
    }

    pub fn close(&self) -> Result<(), KernelError> {
        self.bus.request(CloseZeroCalibration)
    }

    /// Opens, starts and measures channels until `count` are accepted.
    pub fn calibrate_channels(&self, count: usize) {
        self.open().unwrap();
        self.start().unwrap();
        for channel in 0..count {
            self.measure().unwrap();
            let next = if channel + 1 == 4 {
                ZeroStep::Done
            } else {
                ZeroStep::Channel
            };
            self.until_step(next);
        }
    }
}

fn install(bus: &Bus, acquisition: &Arc<Acquisition>) {
    let held = Arc::clone(acquisition);
    bus.handle(move |request: HoldInstrument| {
        held.holds.lock().unwrap().push(request.on);
        let refusal = held.behaviour.lock().unwrap().refuse_hold.clone();
        match refusal.filter(|_| request.on) {
            Some(reason) => Err(KernelError::recoverable(reason)),
            None => Ok(()),
        }
    })
    .unwrap();
    let probing = Arc::clone(acquisition);
    let events = bus.clone();
    bus.handle(move |request: CaptureProbe| {
        probing.probes.lock().unwrap().push(request.clone());
        if probing.behaviour.lock().unwrap().answers {
            events.publish(ProbeCaptured {
                id: request.id,
                frames: probing.frames_for(&request),
            });
        }
        Ok(())
    })
    .unwrap();
}
