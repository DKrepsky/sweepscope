use std::sync::{Condvar, Mutex, MutexGuard};

use crate::AcquisitionSetup;

#[derive(Default)]
struct Intent {
    stop: bool,
    capturing: bool,
    force: bool,
    pending: Option<(u64, AcquisitionSetup)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureKind {
    Normal,
    Forced,
}

#[derive(Debug, PartialEq)]
pub enum Work {
    Stop,
    Run {
        setup: Option<(u64, AcquisitionSetup)>,
        capture: Option<CaptureKind>,
    },
}

/// What the module wants from one worker: stored by handlers, consumed by the worker thread. Never held across a device call.
#[derive(Default)]
pub struct WorkerControl {
    intent: Mutex<Intent>,
    wake: Condvar,
}

impl WorkerControl {
    pub fn new() -> Self {
        Self::default()
    }

    /// The latest setup wins: an older pending one is replaced.
    pub fn apply_setup(&self, revision: u64, setup: AcquisitionSetup) {
        self.update(|intent| intent.pending = Some((revision, setup)));
    }

    pub fn set_capturing(&self, on: bool) {
        self.update(|intent| {
            intent.capturing = on;
            intent.force &= on;
        });
    }

    /// Ignored while not capturing.
    pub fn force(&self) {
        self.update(|intent| intent.force = intent.capturing);
    }

    pub fn stop(&self) {
        self.update(|intent| intent.stop = true);
    }

    pub fn is_capturing(&self) -> bool {
        self.lock().capturing
    }

    /// Blocks until there is something to do. A capture needs a configured device: `configured` says whether one
    /// setup was applied before.
    pub fn next_work(&self, configured: bool) -> Work {
        let mut intent = self.lock();
        loop {
            if intent.stop {
                return Work::Stop;
            }
            let setup = intent.pending.take();
            let can_capture = intent.capturing && (configured || setup.is_some());
            if setup.is_some() || can_capture {
                let capture = can_capture.then(|| {
                    if std::mem::take(&mut intent.force) {
                        CaptureKind::Forced
                    } else {
                        CaptureKind::Normal
                    }
                });
                return Work::Run { setup, capture };
            }
            intent = self.wake.wait(intent).unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    fn update(&self, change: impl FnOnce(&mut Intent)) {
        change(&mut self.lock());
        self.wake.notify_all();
    }

    fn lock(&self) -> MutexGuard<'_, Intent> {
        self.intent.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::{ChannelSetup, Coupling, Slope, Sweep, TriggerKind, TriggerSetup};

    fn setup(time_base_index: u8) -> AcquisitionSetup {
        let channel = ChannelSetup {
            enabled: true,
            range_index: 0,
            position_divs: 0.0,
            coupling: Coupling::Dc,
            bandwidth_limit: false,
        };
        AcquisitionSetup {
            channels: [channel.clone(), channel.clone(), channel.clone(), channel],
            time_base_index,
            trigger: TriggerSetup {
                source: 0,
                level_volts: 0.0,
                slope: Slope::Rising,
                sweep: Sweep::Auto,
                horizontal_position_percent: 50.0,
                kind: TriggerKind::Edge,
            },
        }
    }

    #[test]
    fn given_a_burst_of_five_setups_when_the_worker_asks_for_work_then_only_the_last_is_returned() {
        let control = WorkerControl::new();
        for revision in 1..=5 {
            control.apply_setup(revision, setup(revision as u8));
        }

        let work = control.next_work(false);

        assert_eq!(
            work,
            Work::Run {
                setup: Some((5, setup(5))),
                capture: None
            }
        );
    }

    #[test]
    fn given_a_stop_request_when_the_worker_asks_for_work_then_it_gets_stop_before_anything_else() {
        let control = WorkerControl::new();
        control.apply_setup(1, setup(0));
        control.set_capturing(true);

        control.stop();

        assert_eq!(control.next_work(true), Work::Stop);
    }

    #[test]
    fn given_capturing_on_and_a_configured_device_when_work_is_asked_then_a_normal_capture_is_returned() {
        let control = WorkerControl::new();
        control.set_capturing(true);

        assert_eq!(
            control.next_work(true),
            Work::Run {
                setup: None,
                capture: Some(CaptureKind::Normal)
            }
        );
    }

    #[test]
    fn given_a_force_while_capturing_when_work_is_asked_twice_then_only_the_first_capture_is_forced() {
        let control = WorkerControl::new();
        control.set_capturing(true);
        control.force();

        let first = control.next_work(true);
        let second = control.next_work(true);

        assert!(matches!(
            first,
            Work::Run {
                capture: Some(CaptureKind::Forced),
                ..
            }
        ));
        assert!(matches!(
            second,
            Work::Run {
                capture: Some(CaptureKind::Normal),
                ..
            }
        ));
    }

    #[test]
    fn given_a_force_while_not_capturing_when_capturing_starts_then_the_first_capture_is_normal() {
        let control = WorkerControl::new();
        control.force();

        control.set_capturing(true);

        assert!(matches!(
            control.next_work(true),
            Work::Run {
                capture: Some(CaptureKind::Normal),
                ..
            }
        ));
    }

    #[test]
    fn given_capturing_without_any_setup_when_a_setup_arrives_then_it_is_applied_and_a_capture_follows() {
        let control = Arc::new(WorkerControl::new());
        control.set_capturing(true);
        let waiting = Arc::clone(&control);
        let (started, worker_started) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            started.send(()).unwrap();
            waiting.next_work(false)
        });
        worker_started.recv().unwrap();

        control.apply_setup(1, setup(0));

        let work = worker.join().unwrap();
        assert_eq!(
            work,
            Work::Run {
                setup: Some((1, setup(0))),
                capture: Some(CaptureKind::Normal)
            }
        );
    }
}
