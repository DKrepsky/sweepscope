#[path = "domain_support/mod.rs"]
mod domain_support;
mod module_support;

use module_support::{expect_recoverable, ready_rig};
use sweepscope_acquisition::{SetTrigger, TriggerKindName, TriggerPatch};
use sweepscope_kernel::contracts::{PulseCondition, TriggerKind};

fn pulse(text: &str) -> SetTrigger {
    SetTrigger {
        patch: TriggerPatch {
            kind: Some(TriggerKindName::Pulse),
            condition: Some(PulseCondition::Greater),
            width_text: Some(text.to_owned()),
            ..TriggerPatch::default()
        },
    }
}

#[test]
fn given_a_pulse_width_text_when_the_trigger_is_set_then_the_device_gets_the_width_in_seconds() {
    let rig = ready_rig();

    rig.bus.request(pulse("500 us")).unwrap();

    let sent = rig.device.applies().last().unwrap().1.clone();
    assert_eq!(
        sent.trigger.kind,
        TriggerKind::Pulse {
            width_s: 0.0005,
            condition: PulseCondition::Greater
        }
    );
    let state = rig.snapshot().instrument.unwrap().trigger;
    assert_eq!(state.kind, TriggerKindName::Pulse);
}

#[test]
fn given_a_bad_width_text_when_the_trigger_is_set_then_the_request_is_rejected_and_nothing_is_sent() {
    let rig = ready_rig();
    rig.bus.request(pulse("500 us")).unwrap();
    let sent = rig.device.applies().len();
    let before = rig.snapshot().instrument;

    let message = expect_recoverable(rig.bus.request(pulse("abc")));

    assert_eq!(
        message,
        "Invalid width. Enter a time such as 500 \u{b5}s. Applied: 500 \u{b5}s."
    );
    assert_eq!(rig.device.applies().len(), sent);
    assert_eq!(rig.snapshot().instrument, before);
}
