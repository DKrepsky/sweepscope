//! Calibration module: six request handlers and one thread around the pure domain (ADR 0021 item 7).
//!
//! Create it before the device module: it learns the connection from `ConnectionChanged` events and cannot ask for it.
//! Handlers return at once; the sweep advances on the thread as probes come back.

mod core;
mod probe;
mod stamp;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::thread::JoinHandle;
use std::time::Duration;

use sweepscope_kernel::contracts::{ConnectionChanged, ProbeCaptured};
use sweepscope_kernel::{Bus, Clock, KernelError, Storage, Subscription};

use self::core::{Core, Environment};
use crate::{
    CloseZeroCalibration, ContinueZeroChannel, GetZeroCalibration, MeasureZeroChannel, OpenZeroCalibration,
    StartZeroCalibration,
};

/// How long the thread waits for a probe before it looks at the deadlines and the connection.
const TICK: Duration = Duration::from_millis(20);

pub struct CalibrationDeps {
    pub bus: Bus,
    pub clock: Arc<dyn Clock>,
    /// Holds `zero-offsets.json`.
    pub storage: Arc<dyn Storage>,
    pub unix_time_us: fn() -> f64,
}

pub struct CalibrationModule {
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

type Shared = Mutex<Core>;

fn locked(shared: &Shared) -> MutexGuard<'_, Core> {
    shared.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl CalibrationModule {
    pub fn new(deps: CalibrationDeps) -> Result<Self, KernelError> {
        let connections = deps.bus.subscribe::<ConnectionChanged>();
        let probes = deps.bus.subscribe::<ProbeCaptured>();
        let shared = Arc::new(Mutex::new(Core::new(Environment {
            bus: deps.bus.clone(),
            clock: deps.clock,
            storage: deps.storage,
            unix_time_us: deps.unix_time_us,
        })));
        register(&deps.bus, &shared)?;
        let stop = Arc::new(AtomicBool::new(false));
        let worker = std::thread::Builder::new()
            .name("calibration".into())
            .spawn({
                let (shared, stop) = (Arc::clone(&shared), Arc::clone(&stop));
                move || run(&shared, &Inputs { connections, probes }, &stop)
            })
            .map_err(|e| KernelError::fatal(format!("cannot start the calibration thread: {e}")))?;
        Ok(Self {
            stop,
            worker: Some(worker),
        })
    }
}

impl Drop for CalibrationModule {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// The handlers hold the state weakly: the bus lives as long as the application, the module may not.
fn register(bus: &Bus, shared: &Arc<Shared>) -> Result<(), KernelError> {
    let weak = Arc::downgrade(shared);
    on(bus, &weak, |core, _: OpenZeroCalibration| core.open())?;
    on(bus, &weak, |core, _: StartZeroCalibration| core.start())?;
    on(bus, &weak, |core, _: MeasureZeroChannel| core.measure())?;
    on(bus, &weak, |core, _: ContinueZeroChannel| core.continue_channel())?;
    on(bus, &weak, |core, _: CloseZeroCalibration| core.close())?;
    bus.handle(move |_: GetZeroCalibration| {
        let shared = weak.upgrade().ok_or_else(gone)?;
        let state = locked(&shared).state();
        Ok(state)
    })
}

fn on<R>(
    bus: &Bus,
    shared: &Weak<Shared>,
    run: impl Fn(&mut Core, R) -> Result<(), KernelError> + Send + Sync + 'static,
) -> Result<(), KernelError>
where
    R: sweepscope_kernel::Request<Response = ()>,
{
    let shared = shared.clone();
    bus.handle(move |request: R| {
        let shared = shared.upgrade().ok_or_else(gone)?;
        let mut core = locked(&shared);
        run(&mut core, request)
    })
}

fn gone() -> KernelError {
    KernelError::fatal("the calibration module is shut down")
}

struct Inputs {
    connections: Subscription<ConnectionChanged>,
    probes: Subscription<ProbeCaptured>,
}

/// Connection changes go first, so a probe is judged against the newest connection state.
fn run(shared: &Shared, inputs: &Inputs, stop: &AtomicBool) {
    while !stop.load(Ordering::Acquire) {
        for change in inputs.connections.drain() {
            locked(shared).connection(change);
        }
        match inputs.probes.recv_timeout(TICK) {
            Some(probe) => locked(shared).probe_captured(probe),
            None => locked(shared).tick(),
        }
    }
}
