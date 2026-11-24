//! Consumes `FrameCaptured` latest-wins, keeps the measurements slice, announces changes (ADR 0014).
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::Duration;

use sweepscope_kernel::contracts::FrameCaptured;
use sweepscope_kernel::{Bus, Clock, KernelError, Subscription};

use crate::alias::AliasWatch;
use crate::channel::measure_frame;
use crate::messages::{GetMeasurements, MeasurementsChanged};
use crate::slice::Slice;

const POLL: Duration = Duration::from_millis(20);

pub struct MeasurementDeps {
    pub bus: Bus,
    pub clock: Arc<dyn Clock>,
}

pub struct MeasurementModule {
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl MeasurementModule {
    pub fn new(deps: MeasurementDeps) -> Result<Self, KernelError> {
        let frames = deps.bus.subscribe_latest::<FrameCaptured>();
        let shared = Arc::new(Shared {
            bus: deps.bus,
            clock: deps.clock,
            slice: Mutex::new(Slice::new()),
        });
        let reader = Arc::clone(&shared);
        shared
            .bus
            .handle(move |_: GetMeasurements| Ok(reader.slice().state()))?;
        let stop = Arc::new(AtomicBool::new(false));
        let worker = std::thread::Builder::new()
            .name("measurement".into())
            .spawn({
                let stop = Arc::clone(&stop);
                move || run(&shared, &frames, &stop)
            })
            .map_err(|e| KernelError::fatal(format!("cannot start the measurement thread: {e}")))?;
        Ok(Self {
            stop,
            worker: Some(worker),
        })
    }
}

impl Drop for MeasurementModule {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn run(shared: &Shared, frames: &Subscription<FrameCaptured>, stop: &AtomicBool) {
    let mut alias = AliasWatch::default();
    while !stop.load(Ordering::Acquire) {
        if let Some(captured) = frames.recv_timeout(POLL) {
            shared.measure(&captured, &mut alias);
        }
        shared.announce_held_back();
    }
}

struct Shared {
    bus: Bus,
    clock: Arc<dyn Clock>,
    slice: Mutex<Slice>,
}

impl Shared {
    fn slice(&self) -> MutexGuard<'_, Slice> {
        self.slice.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn measure(&self, captured: &FrameCaptured, alias: &mut AliasWatch) {
        let derived = captured.derived.as_deref();
        let Some(channels) = measure_frame(&captured.frame, &captured.context, derived) else {
            return;
        };
        let warnings = alias.evaluate(&channels, &captured.frame, &captured.context);
        let announced = self.slice().update(channels, warnings, self.clock.now());
        if let Some(state) = announced {
            self.bus.publish(MeasurementsChanged(state));
        }
    }

    fn announce_held_back(&self) {
        let announced = self.slice().announce_if_due(self.clock.now());
        if let Some(state) = announced {
            self.bus.publish(MeasurementsChanged(state));
        }
    }
}
