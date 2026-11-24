use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use sweepscope_display::{stamp_sent, DisplayFrameReady, DisplayRollItem};
use sweepscope_kernel::{Bus, Event, KernelError, Subscription};
use sweepscope_spectrum::SpectrumFrameReady;
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

/// An encoded-bytes event the shell forwards on a `Channel`, and the delivery rule of its stream.
pub trait StreamEvent: Event {
    const NAME: &'static str;
    /// Ordered streams see every item (bounded queue, the loss rule is the consumer's gap); the others keep the newest.
    const ORDERED: bool;
    /// Writes the sent time into the frame header (offset 16) when the format has one.
    fn stamp(bytes: &mut [u8], sent_us: f64);
    fn into_bytes(self) -> Arc<Vec<u8>>;
}

impl StreamEvent for DisplayFrameReady {
    const NAME: &'static str = "frame";
    const ORDERED: bool = false;
    fn stamp(bytes: &mut [u8], sent_us: f64) {
        stamp_sent(bytes, sent_us);
    }
    fn into_bytes(self) -> Arc<Vec<u8>> {
        self.bytes
    }
}

impl StreamEvent for SpectrumFrameReady {
    const NAME: &'static str = "spectrum";
    const ORDERED: bool = false;
    fn stamp(bytes: &mut [u8], sent_us: f64) {
        sweepscope_spectrum::stamp_sent(bytes, sent_us);
    }
    fn into_bytes(self) -> Arc<Vec<u8>> {
        self.bytes
    }
}

/// `SSL1` items carry a sequence number and no sent time.
impl StreamEvent for DisplayRollItem {
    const NAME: &'static str = "roll";
    const ORDERED: bool = true;
    fn stamp(_: &mut [u8], _: f64) {}
    fn into_bytes(self) -> Arc<Vec<u8>> {
        self.bytes
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

/// One stream of the window per event type (ADR 0011 item 4): opening again replaces the previous forwarder.
pub struct Streams<E: StreamEvent> {
    active: Mutex<Option<Forwarder>>,
    event: PhantomData<fn(E)>,
}

pub type FrameStreams = Streams<DisplayFrameReady>;
pub type RollStreams = Streams<DisplayRollItem>;
pub type SpectrumStreams = Streams<SpectrumFrameReady>;

impl<E: StreamEvent> Default for Streams<E> {
    fn default() -> Self {
        Self {
            active: Mutex::new(None),
            event: PhantomData,
        }
    }
}

fn subscribe<E: StreamEvent>(bus: &Bus) -> Subscription<E> {
    if E::ORDERED {
        bus.subscribe::<E>()
    } else {
        bus.subscribe_latest::<E>()
    }
}

impl<E: StreamEvent> Streams<E> {
    pub fn open(&self, bus: &Bus, sink: impl FrameSink, unix_time_us: fn() -> f64) -> Result<(), KernelError> {
        let frames = subscribe::<E>(bus);
        let stop = Arc::new(AtomicBool::new(false));
        let thread = std::thread::Builder::new()
            .name(format!("{}-forwarder", E::NAME))
            .spawn({
                let stop = Arc::clone(&stop);
                move || {
                    while !stop.load(Ordering::Acquire) {
                        let Some(ready) = frames.recv_timeout(POLL) else {
                            continue;
                        };
                        let mut bytes = Arc::unwrap_or_clone(ready.into_bytes());
                        E::stamp(&mut bytes, unix_time_us());
                        if sink.send(bytes).is_err() {
                            break;
                        }
                    }
                }
            })
            .map_err(|error| KernelError::fatal(format!("cannot start the {} forwarder: {error}", E::NAME)))?;
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

    fn roll_item(marker: u8) -> DisplayRollItem {
        let mut bytes = vec![0u8; 48];
        bytes[40] = marker;
        DisplayRollItem { bytes: Arc::new(bytes) }
    }

    fn markers(sink: &Collector) -> Vec<u8> {
        sink.frames.lock().unwrap().iter().map(|bytes| bytes[40]).collect()
    }

    #[test]
    fn given_a_roll_stream_when_items_are_published_in_a_burst_then_every_item_arrives_in_order() {
        let bus = Bus::new();
        let streams = RollStreams::default();
        let sink = Collector::default();
        streams.open(&bus, sink.clone(), now).expect("open");
        let expected: Vec<u8> = (0..=199).collect();

        for marker in &expected {
            bus.publish(roll_item(*marker));
        }

        assert!(eventually(WAIT, || sink.count() == expected.len()));
        assert_eq!(markers(&sink), expected);
    }

    #[test]
    fn given_a_roll_stream_when_an_item_is_forwarded_then_its_bytes_are_not_stamped() {
        let bus = Bus::new();
        let streams = RollStreams::default();
        let sink = Collector::default();
        streams.open(&bus, sink.clone(), now).expect("open");

        bus.publish(roll_item(3));

        assert!(eventually(WAIT, || sink.count() == 1));
        assert_eq!(sink.last().expect("item")[16..24], [0u8; 8]);
    }

    #[test]
    fn given_a_spectrum_stream_when_a_frame_is_published_then_the_sink_gets_it_stamped_with_the_sent_time() {
        let bus = Bus::new();
        let streams = SpectrumStreams::default();
        let sink = Collector::default();
        streams.open(&bus, sink.clone(), now).expect("open");

        bus.publish(SpectrumFrameReady {
            bytes: Arc::new(vec![0u8; 48]),
        });

        assert!(eventually(WAIT, || sink.count() == 1));
        assert_eq!(sink.last().expect("frame")[16..24], SENT_AT.to_le_bytes());
    }
}
