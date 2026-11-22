use std::time::{Duration, Instant};

use sweepscope_kernel::testing::FakeClock;
use sweepscope_kernel::{Clock, SystemClock};

#[test]
fn given_a_fake_clock_when_it_sleeps_then_time_advances_by_exactly_that_duration_without_waiting() {
    let clock = FakeClock::default();
    let start = clock.now();
    let wall = Instant::now();

    clock.sleep(Duration::from_secs(3600));

    assert_eq!(clock.now() - start, Duration::from_secs(3600));
    assert!(wall.elapsed() < Duration::from_secs(1));
}

#[test]
fn given_a_fake_clock_when_a_clone_sleeps_then_the_original_sees_the_same_time() {
    let clock = FakeClock::default();
    let start = clock.now();

    clock.clone().sleep(Duration::from_millis(20));

    assert_eq!(clock.now() - start, Duration::from_millis(20));
}

#[test]
fn given_the_system_clock_when_it_sleeps_then_at_least_that_much_time_passes() {
    let clock = SystemClock;
    let start = clock.now();

    clock.sleep(Duration::from_millis(5));

    assert!(clock.now() - start >= Duration::from_millis(5));
}
