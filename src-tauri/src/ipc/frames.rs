use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use sweepscope_display::{stamp_sent, DisplayFrameReady};
use sweepscope_kernel::{Bus, KernelError};
use tauri::ipc::{Channel, InvokeResponseBody};

const POLL: Duration = Duration::from_millis(50);

/// The receiving end is gone; the forwarder stops.
#[derive(Debug)]
pub struct SinkClosed;

/// Where encoded frames go; the webview `Channel` in production, a collector in tests.
pub trait FrameSink: Send + 'static {
    fn send(&self, bytes: Vec<u8>) -> Result<(), SinkClosed>;
}

impl FrameSink for Channel<InvokeResponseBody> {
    fn send(&self, bytes: Vec<u8>) -> Result<(), SinkClosed> {
        Channel::send(self, InvokeResponseBody::Raw(bytes)).map_err(|_| SinkClosed)
    }
}

struct Forwarder {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Drop for Forwarder {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// The one frame stream of the window (ADR 0011 item 4): opening again replaces the previous forwarder.
#[derive(Default)]
pub struct FrameStreams {
    active: Mutex<Option<Forwarder>>,
}

impl FrameStreams {
    pub fn open(&self, bus: &Bus, sink: impl FrameSink, unix_time_us: fn() -> f64) -> Result<(), KernelError> {
        let frames = bus.subscribe_latest::<DisplayFrameReady>();
        let stop = Arc::new(AtomicBool::new(false));
        let thread = std::thread::Builder::new()
            .name("frame-forwarder".into())
            .spawn({
                let stop = Arc::clone(&stop);
                move || {
                    while !stop.load(Ordering::Acquire) {
                        let Some(ready) = frames.recv_timeout(POLL) else {
                            continue;
                        };
                        let mut bytes = Arc::unwrap_or_clone(ready.bytes);
                        stamp_sent(&mut bytes, unix_time_us());
                        if sink.send(bytes).is_err() {
                            break;
                        }
                    }
                }
            })
            .map_err(|error| KernelError::fatal(format!("cannot start the frame forwarder: {error}")))?;
        let previous = self.slot().replace(Forwarder {
            stop,
            thread: Some(thread),
        });
        drop(previous);
        Ok(())
    }

    pub fn close(&self) {
        let previous = self.slot().take();
        drop(previous);
    }

    fn slot(&self) -> std::sync::MutexGuard<'_, Option<Forwarder>> {
        self.active.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use sweepscope_kernel::testing::eventually;

    use super::*;

    const WAIT: Duration = Duration::from_secs(5);
    const SENT_AT: f64 = 1234.5;

    #[derive(Clone, Default)]
    struct Collector {
        frames: Arc<Mutex<Vec<Vec<u8>>>>,
        closed: bool,
    }

    impl Collector {
        fn count(&self) -> usize {
            self.frames.lock().unwrap().len()
        }

        fn last(&self) -> Option<Vec<u8>> {
            self.frames.lock().unwrap().last().cloned()
        }
    }

    impl FrameSink for Collector {
        fn send(&self, bytes: Vec<u8>) -> Result<(), SinkClosed> {
            self.frames.lock().unwrap().push(bytes);
            if self.closed {
                return Err(SinkClosed);
            }
            Ok(())
        }
    }

    fn frame(marker: u8) -> DisplayFrameReady {
        let mut bytes = vec![0u8; 48];
        bytes[40] = marker;
        DisplayFrameReady { bytes: Arc::new(bytes) }
    }

    fn now() -> f64 {
        SENT_AT
    }

    #[test]
    fn given_an_open_stream_when_a_frame_is_published_then_the_sink_gets_it_stamped_with_the_sent_time() {
        let bus = Bus::new();
        let streams = FrameStreams::default();
        let sink = Collector::default();
        streams.open(&bus, sink.clone(), now).expect("open");

        bus.publish(frame(9));

        assert!(eventually(WAIT, || sink.count() == 1));
        let bytes = sink.last().expect("frame");
        assert_eq!(bytes[16..24], SENT_AT.to_le_bytes());
        assert_eq!(bytes[40], 9);
        assert_eq!(bytes[..16], [0u8; 16]);
    }

    #[test]
    fn given_two_frames_published_in_a_row_when_forwarded_then_the_newest_arrives_last() {
        let bus = Bus::new();
        let streams = FrameStreams::default();
        let sink = Collector::default();
        streams.open(&bus, sink.clone(), now).expect("open");

        bus.publish(frame(1));
        bus.publish(frame(2));

        assert!(eventually(WAIT, || sink.last().is_some_and(|b| b[40] == 2)));
    }

    #[test]
    fn given_a_closed_stream_when_a_frame_is_published_then_the_sink_gets_nothing() {
        let bus = Bus::new();
        let streams = FrameStreams::default();
        let sink = Collector::default();
        streams.open(&bus, sink.clone(), now).expect("open");

        streams.close();
        bus.publish(frame(1));

        assert_eq!(sink.count(), 0);
    }

    #[test]
    fn given_a_stream_when_opened_again_then_only_the_new_sink_receives_frames() {
        let bus = Bus::new();
        let streams = FrameStreams::default();
        let first = Collector::default();
        let second = Collector::default();
        streams.open(&bus, first.clone(), now).expect("open first");
        streams.open(&bus, second.clone(), now).expect("open second");

        bus.publish(frame(1));

        assert!(eventually(WAIT, || second.count() == 1));
        assert_eq!(first.count(), 0);
    }

    #[test]
    fn given_a_sink_that_reports_closed_when_frames_keep_coming_then_the_forwarder_stops_sending() {
        let bus = Bus::new();
        let streams = FrameStreams::default();
        let sink = Collector {
            closed: true,
            ..Collector::default()
        };
        streams.open(&bus, sink.clone(), now).expect("open");
        bus.publish(frame(1));
        assert!(eventually(WAIT, || sink.count() == 1));

        bus.publish(frame(2));
        streams.close();

        assert_eq!(sink.count(), 1);
    }
}
