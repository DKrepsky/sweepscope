//! Alone in this file on purpose: it counts the threads of the whole process.
mod module_support;

use std::sync::atomic::Ordering;

use sweepscope_kernel::contracts::ConnectionState;

use module_support::*;

fn thread_count() -> usize {
    std::fs::read_dir("/proc/self/task").unwrap().count()
}

#[test]
fn given_20_unplug_and_replug_cycles_when_they_finish_then_the_state_is_ready_and_no_thread_is_left_over() {
    let mut simulator = Source::plugged();
    let rig = rig(None, Some(&mut simulator));
    rig.wait_for_state(ConnectionState::Ready);
    let baseline = thread_count();

    for _ in 0..20 {
        simulator.control.unplug();
        rig.wait_for_state(ConnectionState::Disconnected);
        simulator.control.plug_in();
        rig.wait_for_state(ConnectionState::Ready);
    }

    assert!(
        wait_until(|| thread_count() == baseline),
        "{} threads, baseline {baseline}",
        thread_count()
    );
    assert_eq!(simulator.hooks.live_devices.load(Ordering::SeqCst), 1);
    assert_eq!(simulator.hooks.opened.load(Ordering::SeqCst), 21);
}
