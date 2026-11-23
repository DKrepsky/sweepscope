use std::thread;
use std::time::{Duration, Instant};
use sweepscope_kernel::testing::{eventually, EventRecorder, FakeClock};
use sweepscope_kernel::{Bus, Clock, ErrorCategory, Event, Request};

#[derive(Clone, Debug, PartialEq)]
struct Ping(u32);
impl Event for Ping {}

#[derive(Clone, Debug, PartialEq)]
struct Pong;
impl Event for Pong {}

struct Double(u32);
impl Request for Double {
    type Response = u32;
}

#[test]
fn given_a_subscriber_when_events_are_published_then_it_receives_each_once_in_order() {
    let bus = Bus::new();
    let subscription = bus.subscribe::<Ping>();

    bus.publish(Ping(1));
    bus.publish(Ping(2));

    assert_eq!(subscription.drain(), vec![Ping(1), Ping(2)]);
    assert_eq!(subscription.try_next(), None);
}

#[test]
fn given_two_subscribers_of_one_type_when_published_then_both_receive_it_and_other_types_do_not() {
    let bus = Bus::new();
    let first = EventRecorder::<Ping>::new(&bus);
    let second = EventRecorder::<Ping>::new(&bus);
    let other = EventRecorder::<Pong>::new(&bus);

    bus.publish(Ping(7));

    assert_eq!(first.events(), vec![Ping(7)]);
    assert_eq!(second.events(), vec![Ping(7)]);
    assert!(other.events().is_empty());
}

#[test]
fn given_a_dropped_subscription_when_published_then_other_subscribers_still_receive() {
    let bus = Bus::new();
    drop(bus.subscribe::<Ping>());
    let alive = bus.subscribe::<Ping>();

    bus.publish(Ping(1));

    assert_eq!(alive.try_next(), Some(Ping(1)));
}

#[test]
fn given_a_full_subscriber_queue_when_more_events_arrive_then_the_oldest_drop_and_lag_counts_them() {
    let bus = Bus::new();
    let subscription = bus.subscribe::<Ping>();

    for n in 0..260 {
        bus.publish(Ping(n));
    }

    assert_eq!(subscription.lagged(), 4);
    assert_eq!(subscription.try_next(), Some(Ping(4)));
}

#[test]
fn given_a_registered_handler_when_a_request_is_sent_then_the_typed_response_returns() {
    let bus = Bus::new();
    bus.handle(|Double(n)| Ok(n * 2)).unwrap();

    assert_eq!(bus.request(Double(21)), Ok(42));
}

#[test]
fn given_no_handler_when_a_request_is_sent_then_no_handler_error_names_the_type() {
    let error = Bus::new().request(Double(1)).unwrap_err();

    assert_eq!(error.category, ErrorCategory::Fatal);
    assert!(error.message.contains("Double"), "{}", error.message);
}

#[test]
fn given_a_handler_when_a_second_registers_then_it_is_refused_and_the_first_stays() {
    let bus = Bus::new();
    bus.handle(|Double(n)| Ok(n * 2)).unwrap();

    let second = bus.handle(|Double(n)| Ok(n * 3));

    assert!(second.is_err());
    assert_eq!(bus.request(Double(1)), Ok(2));
}

#[test]
fn given_a_fake_clock_when_advanced_then_now_reflects_it() {
    let clock = FakeClock::default();
    let start = clock.now();

    clock.advance(Duration::from_secs(3));

    assert_eq!(clock.now() - start, Duration::from_secs(3));
}

#[test]
fn given_a_subscriber_waiting_when_an_event_is_published_from_another_thread_then_it_returns_before_the_timeout() {
    let bus = Bus::new();
    let subscription = bus.subscribe::<Ping>();
    let publisher = bus.clone();

    let handle = thread::spawn(move || {
        publisher.publish(Ping(5));
    });
    let received = subscription.recv_timeout(Duration::from_secs(5));
    handle.join().unwrap();

    assert_eq!(received, Some(Ping(5)));
}

#[test]
fn given_a_queued_event_when_recv_timeout_is_called_then_it_returns_at_once() {
    let bus = Bus::new();
    let subscription = bus.subscribe::<Ping>();
    bus.publish(Ping(1));

    assert_eq!(subscription.recv_timeout(Duration::ZERO), Some(Ping(1)));
}

#[test]
fn given_no_event_when_recv_timeout_elapses_then_it_returns_none_after_waiting() {
    let subscription = Bus::new().subscribe::<Ping>();
    let started = Instant::now();

    let received = subscription.recv_timeout(Duration::from_millis(40));

    assert_eq!(received, None);
    assert!(started.elapsed() >= Duration::from_millis(40));
}

#[test]
fn given_a_latest_subscription_when_three_events_are_published_then_only_the_last_is_received_and_two_lagged() {
    let bus = Bus::new();
    let subscription = bus.subscribe_latest::<Ping>();

    bus.publish(Ping(1));
    bus.publish(Ping(2));
    bus.publish(Ping(3));

    assert_eq!(subscription.lagged(), 2);
    assert_eq!(subscription.drain(), vec![Ping(3)]);
}

#[test]
fn given_a_latest_subscription_when_it_waits_then_it_wakes_on_publish() {
    let bus = Bus::new();
    let subscription = bus.subscribe_latest::<Ping>();
    let publisher = bus.clone();

    let handle = thread::spawn(move || {
        publisher.publish(Ping(9));
    });
    let received = subscription.recv_timeout(Duration::from_secs(5));
    handle.join().unwrap();

    assert_eq!(received, Some(Ping(9)));
}

#[test]
fn given_a_condition_that_becomes_true_when_eventually_polls_then_it_returns_true() {
    let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let setter = flag.clone();
    let handle = thread::spawn(move || {
        setter.store(true, std::sync::atomic::Ordering::SeqCst);
    });

    let reached = eventually(Duration::from_secs(5), || {
        flag.load(std::sync::atomic::Ordering::SeqCst)
    });
    handle.join().unwrap();

    assert!(reached);
}

#[test]
fn given_a_condition_that_stays_false_when_eventually_times_out_then_it_returns_false() {
    assert!(!eventually(Duration::from_millis(30), || false));
}
