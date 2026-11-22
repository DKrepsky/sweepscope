use std::time::Duration;
use sweepscope_kernel::testing::{EventRecorder, FakeClock};
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
