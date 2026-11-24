//! Smoke tests for a real 6254BC. Ignored by default; the human runs them with
//! `cargo test -p sweepscope-device --test hantek_hardware -- --ignored --nocapture --test-threads=1`.

use std::time::Duration;

use sweepscope_device::{
    AcquisitionSetup, CatalogEvent, ChannelSetup, Coupling, DeviceCatalog, HantekCatalog, ScopeDevice, Slope, Sweep,
    TriggerKind, TriggerSetup,
};

const ATTACH_WAIT: Duration = Duration::from_secs(10);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(3);

fn open_first_scope() -> Box<dyn ScopeDevice> {
    let catalog = HantekCatalog::start().expect("start the hotplug monitor");
    let events = catalog.events();
    let id = loop {
        match events.recv_timeout(ATTACH_WAIT).expect("no scope attached within 10 s") {
            CatalogEvent::Attached(id) => break id,
            CatalogEvent::Detached(_) => {}
        }
    };
    catalog.open(&id).expect("open the attached scope")
}

fn setup(enabled_channels: usize) -> AcquisitionSetup {
    let channel = |index: usize| ChannelSetup {
        enabled: index < enabled_channels,
        range_index: 8,
        position_divs: 0.0,
        coupling: Coupling::Dc,
        bandwidth_limit: false,
    };
    AcquisitionSetup {
        channels: [channel(0), channel(1), channel(2), channel(3)],
        time_base_index: 15,
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

fn capture_and_print(device: &mut dyn ScopeDevice, enabled_channels: usize) {
    device.configure(&setup(enabled_channels)).expect("configure");
    let frame = device.capture(CAPTURE_TIMEOUT).expect("capture");
    let lengths: Vec<usize> = frame
        .channels
        .iter()
        .map(|channel| channel.as_ref().map_or(0, |data| data.codes.len()))
        .collect();
    println!(
        "{enabled_channels} channel(s): lengths {lengths:?}, interval {} s, trigger {:?}",
        frame.sample_interval_s, frame.trigger
    );
    assert_eq!(
        lengths.iter().filter(|length| **length == 4096).count(),
        enabled_channels
    );
}

#[test]
#[ignore = "needs a connected Hantek 6254BC"]
fn given_hardware_when_opened_then_the_identity_is_reported() {
    let device = open_first_scope();

    println!("{:#?}", device.identity());

    assert!(!device.identity().simulated);
}

#[test]
#[ignore = "needs a connected Hantek 6254BC"]
fn given_hardware_with_one_channel_when_captured_then_one_record_arrives() {
    capture_and_print(open_first_scope().as_mut(), 1);
}

#[test]
#[ignore = "needs a connected Hantek 6254BC"]
fn given_hardware_with_four_channels_when_captured_then_four_records_arrive() {
    capture_and_print(open_first_scope().as_mut(), 4);
}
