//! Computes spectra of the latest frame while the FFT view is open, keeps the slice, publishes `SSS1` frames (ADR 0018).
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::Duration;

use sweepscope_kernel::contracts::{FrameCaptured, RollCaptured, RollItem};
use sweepscope_kernel::{Bus, Clock, KernelError, Subscription};

use crate::analysis::{analyse, record_of, Analysis};
use crate::axis::{Axis, LinearTop};
use crate::messages::{GetSpectrum, SetSpectrum, SpectrumChanged, SpectrumFrameReady, SpectrumPatch, SpectrumScale};
use crate::slice::Slice;
use crate::state::{build, Settings};
use crate::wire::{encode, ChannelSpectrum, FrameHeader};

const POLL: Duration = Duration::from_millis(20);

pub struct SpectrumDeps {
    pub bus: Bus,
    pub clock: Arc<dyn Clock>,
    pub unix_time_us: fn() -> f64,
}

pub struct SpectrumModule {
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl SpectrumModule {
    pub fn new(deps: SpectrumDeps) -> Result<Self, KernelError> {
        let frames = deps.bus.subscribe_latest::<FrameCaptured>();
        let rolls = deps.bus.subscribe::<RollCaptured>();
        let shared = Arc::new(Shared {
            bus: deps.bus,
            clock: deps.clock,
            unix_time_us: deps.unix_time_us,
            core: Mutex::new(Core::new()),
        });
        let reader = Arc::clone(&shared);
        shared
            .bus
            .handle(move |_: GetSpectrum| Ok(reader.core().slice.state()))?;
        let writer = Arc::clone(&shared);
        shared.bus.handle(move |request: SetSpectrum| {
            writer.change_settings(&request.patch);
            Ok(())
        })?;
        let stop = Arc::new(AtomicBool::new(false));
        let worker = std::thread::Builder::new()
            .name("spectrum".into())
            .spawn({
                let stop = Arc::clone(&stop);
                move || run(&shared, &frames, &rolls, &stop)
            })
            .map_err(|e| KernelError::fatal(format!("cannot start the spectrum thread: {e}")))?;
        Ok(Self {
            stop,
            worker: Some(worker),
        })
    }
}

impl Drop for SpectrumModule {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn run(shared: &Shared, frames: &Subscription<FrameCaptured>, rolls: &Subscription<RollCaptured>, stop: &AtomicBool) {
    while !stop.load(Ordering::Acquire) {
        if let Some(captured) = frames.recv_timeout(POLL) {
            shared.on_frame(captured);
        }
        for roll in rolls.drain() {
            shared.on_roll(&roll.item);
        }
        shared.announce_held_back();
    }
}

struct Core {
    settings: Settings,
    rolling: bool,
    last: Option<FrameCaptured>,
    top: LinearTop,
    slice: Slice,
    sequence: u32,
}

impl Core {
    fn new() -> Self {
        let settings = Settings::default();
        let top = LinearTop::default();
        let initial = build(settings, None, Axis::dbv(), false);
        Self {
            settings,
            rolling: false,
            last: None,
            top,
            slice: Slice::new(initial),
            sequence: 0,
        }
    }
}

struct Shared {
    bus: Bus,
    clock: Arc<dyn Clock>,
    unix_time_us: fn() -> f64,
    core: Mutex<Core>,
}

impl Shared {
    fn core(&self) -> MutexGuard<'_, Core> {
        self.core.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn on_frame(&self, captured: FrameCaptured) {
        let mut core = self.core();
        core.rolling = false;
        core.last = Some(captured);
        self.refresh(&mut core);
    }

    fn on_roll(&self, item: &RollItem) {
        let mut core = self.core();
        let rolling = match item {
            RollItem::Opened => true,
            RollItem::Closed { .. } => false,
            _ => core.rolling,
        };
        if rolling != core.rolling {
            core.rolling = rolling;
            self.refresh(&mut core);
        }
    }

    fn change_settings(&self, patch: &SpectrumPatch) {
        let mut core = self.core();
        let settings = &mut core.settings;
        settings.active = patch.active.unwrap_or(settings.active);
        settings.window = patch.window.unwrap_or(settings.window);
        settings.scale = patch.scale.unwrap_or(settings.scale);
        self.refresh(&mut core);
    }

    fn announce_held_back(&self) {
        let announced = self.core().slice.announce_if_due(self.clock.now());
        if let Some(state) = announced {
            self.bus.publish(SpectrumChanged(state));
        }
    }

    /// Recomputes from the last frame whenever a setting or the frame changed, so a stopped capture redraws too.
    fn refresh(&self, core: &mut Core) {
        let analysis = self.analysis(core);
        let axis = axis_for(core, analysis.as_ref());
        let revision = core
            .last
            .as_ref()
            .and_then(|last| u32::try_from(last.context.setup_revision).ok());
        if let (Some(analysis), Some(_), Some(revision)) = (&analysis, &core.last, revision) {
            let bytes = self.encode_frame(core, analysis, axis, revision);
            core.sequence = core.sequence.wrapping_add(1);
            self.bus.publish(SpectrumFrameReady { bytes: Arc::new(bytes) });
        }
        let record = core.last.as_ref().and_then(|last| record_of(&last.frame));
        let state = build(core.settings, record, axis, core.rolling);
        if let Some(announced) = core.slice.update(state, self.clock.now()) {
            self.bus.publish(SpectrumChanged(announced));
        }
    }

    fn analysis(&self, core: &Core) -> Option<Analysis> {
        let computing = core.settings.active && !core.rolling;
        let last = core.last.as_ref().filter(|_| computing)?;
        analyse(last, core.settings.window)
    }

    fn encode_frame(&self, core: &Core, analysis: &Analysis, axis: Axis, setup_revision: u32) -> Vec<u8> {
        let channels: Vec<ChannelSpectrum> = analysis
            .channels
            .iter()
            .map(|analysed| {
                let placed: Vec<(f32, f32)> = analysed.spectrum.rms.iter().map(|&rms| axis.place(rms)).collect();
                ChannelSpectrum {
                    slot: analysed.slot,
                    peak_bin: analysed.peak_bin.map(|bin| bin as u32),
                    levels: placed.iter().map(|&(level, _)| level).collect(),
                    divisions: placed.iter().map(|&(_, divisions)| divisions).collect(),
                }
            })
            .collect();
        let header = FrameHeader {
            linear: axis.scale == SpectrumScale::Linear,
            window: core.settings.window,
            bin_hz: analysis.record.bin_hz(),
            top: axis.top as f32,
            units_per_div: axis.units_per_div() as f32,
            setup_revision,
        };
        encode(&header, &channels, core.sequence, (self.unix_time_us)())
    }
}

fn axis_for(core: &mut Core, analysis: Option<&Analysis>) -> Axis {
    match core.settings.scale {
        SpectrumScale::Dbv => Axis::dbv(),
        SpectrumScale::Linear => {
            let largest = analysis.map(|a| {
                a.channels
                    .iter()
                    .flat_map(|channel| channel.spectrum.rms.iter().copied())
                    .fold(0.0, f64::max)
            });
            let top = match largest {
                Some(peak) => core.top.follow(peak),
                None => core.top.current(),
            };
            Axis::linear(top)
        }
    }
}
