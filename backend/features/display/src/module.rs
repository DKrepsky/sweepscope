//! Display module: consumes `FrameCaptured` latest-wins, publishes `DisplayFrameReady`, owns the interpolation mode (ADR 0015).
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::Duration;

use sweepscope_kernel::contracts::FrameCaptured;
use sweepscope_kernel::{Bus, KernelError, Subscription};

use crate::messages::{
    DisplayChanged, DisplayFrameReady, DisplayState, GetDisplay, InterpolationMode, SetInterpolation,
};
use crate::mode::automatic_mode;
use crate::wire::encode_in_mode;

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
        let shared = Arc::new(Shared::new(deps));
        register_handlers(&shared)?;
        let stop = Arc::new(AtomicBool::new(false));
        let worker = std::thread::Builder::new()
            .name("display".into())
            .spawn({
                let stop = Arc::clone(&stop);
                move || run(&shared, &frames, &stop)
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

fn register_handlers(shared: &Arc<Shared>) -> Result<(), KernelError> {
    let bus = shared.bus.clone();
    let reader = Arc::clone(shared);
    bus.handle(move |_: GetDisplay| Ok(reader.state()))?;
    let writer = Arc::clone(shared);
    bus.handle(move |request: SetInterpolation| {
        writer.choose(request.mode);
        Ok(())
    })
}

fn run(shared: &Shared, frames: &Subscription<FrameCaptured>, stop: &AtomicBool) {
    while !stop.load(Ordering::Acquire) {
        if let Some(captured) = frames.recv_timeout(POLL) {
            shared.show(captured);
        }
    }
}

struct Shared {
    bus: Bus,
    unix_time_us: fn() -> f64,
    inner: Mutex<Inner>,
}

struct Inner {
    state: DisplayState,
    chosen: Option<InterpolationMode>,
    last: Option<FrameCaptured>,
    sequence: u32,
}

impl Shared {
    fn new(deps: DisplayDeps) -> Self {
        Self {
            bus: deps.bus,
            unix_time_us: deps.unix_time_us,
            inner: Mutex::new(Inner {
                state: DisplayState {
                    revision: 0,
                    interpolation: InterpolationMode::Linear,
                    automatic: true,
                },
                chosen: None,
                last: None,
                sequence: 0,
            }),
        }
    }

    fn inner(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn state(&self) -> DisplayState {
        self.inner().state
    }

    fn show(&self, captured: FrameCaptured) {
        let mut inner = self.inner();
        inner.last = Some(captured);
        self.refresh(&mut inner);
        self.publish_last(&mut inner);
    }

    fn choose(&self, mode: InterpolationMode) {
        let mut inner = self.inner();
        inner.chosen = Some(mode);
        self.refresh(&mut inner);
        self.publish_last(&mut inner);
    }

    /// Recomputes the effective mode from the choice and the newest frame; publishes only a real change.
    fn refresh(&self, inner: &mut Inner) {
        let window_len = inner.last.as_ref().map_or(0, |last| last.context.window_len);
        let next = DisplayState {
            revision: inner.state.revision,
            interpolation: inner.chosen.unwrap_or_else(|| automatic_mode(window_len)),
            automatic: inner.chosen.is_none(),
        };
        if next == inner.state {
            return;
        }
        inner.state = DisplayState {
            revision: next.revision + 1,
            ..next
        };
        self.bus.publish(DisplayChanged(inner.state));
    }

    fn publish_last(&self, inner: &mut Inner) {
        let Some(last) = &inner.last else {
            return;
        };
        let bytes = encode_in_mode(
            &last.frame,
            &last.context,
            inner.sequence,
            (self.unix_time_us)(),
            inner.state.interpolation,
        );
        inner.sequence = inner.sequence.wrapping_add(1);
        self.bus.publish(DisplayFrameReady { bytes: Arc::new(bytes) });
    }
}
