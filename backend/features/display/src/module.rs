//! Display module: consumes `FrameCaptured` latest-wins and publishes `DisplayFrameReady`.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use sweepscope_kernel::contracts::FrameCaptured;
use sweepscope_kernel::{Bus, KernelError, Subscription};

use crate::messages::DisplayFrameReady;
use crate::wire::encode;

const POLL: Duration = Duration::from_millis(50);

pub struct DisplayDeps {
    pub bus: Bus,
    pub unix_time_us: fn() -> f64,
}

pub struct DisplayModule {
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl DisplayModule {
    pub fn new(deps: DisplayDeps) -> Result<Self, KernelError> {
        let frames = deps.bus.subscribe_latest::<FrameCaptured>();
        let stop = Arc::new(AtomicBool::new(false));
        let worker = std::thread::Builder::new()
            .name("display".into())
            .spawn({
                let stop = Arc::clone(&stop);
                move || run(&deps, &frames, &stop)
            })
            .map_err(|e| KernelError::fatal(format!("cannot start the display thread: {e}")))?;
        Ok(Self {
            stop,
            worker: Some(worker),
        })
    }
}

impl Drop for DisplayModule {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn run(deps: &DisplayDeps, frames: &Subscription<FrameCaptured>, stop: &AtomicBool) {
    let mut sequence = 0u32;
    while !stop.load(Ordering::Acquire) {
        let Some(captured) = frames.recv_timeout(POLL) else {
            continue;
        };
        let bytes = encode(&captured.frame, &captured.context, sequence, (deps.unix_time_us)());
        sequence = sequence.wrapping_add(1);
        deps.bus.publish(DisplayFrameReady { bytes: Arc::new(bytes) });
    }
}
