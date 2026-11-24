//! Alone in this file on purpose: it counts the threads of the whole process.
mod module_support;

use std::sync::atomic::Ordering;

use sweepscope_kernel::contracts::{ApplySetup, ConnectionState, SetCapturing};

use module_support::*;

fn thread_count() -> usize {
    std::fs::read_dir("/proc/self/task").unwrap().count()
}

#[test]
fn given_20_rolls_ended_by_unplugging_when_they_finish_then_no_thread_is_left_over() {
    let mut simulator = Source::plugged();
    let rig = rig(None, Some(&mut simulator));
    rig.wait_for_state(ConnectionState::Ready);
    let baseline = thread_count();

    for revision in 1..=20 {
        rig.bus
            .request(ApplySetup {
                setup_revision: revision,
                setup: roll_setup(TB_500MS),
            })
            .unwrap();
        rig.bus.request(SetCapturing { on: true }).unwrap();
        rig.polled.recv_timeout(WAIT).expect("the roll polls");
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
}
