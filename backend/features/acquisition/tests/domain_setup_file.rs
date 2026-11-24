mod domain_support;

use domain_support::{capabilities, enable_all, patch_position, ready_instrument};
use serde_json::{json, Value};
use sweepscope_acquisition::{
    ChannelPatch, Defaults, Instrument, SettingTarget, SetupFile, TriggerKindName, TriggerPatch,
};
use sweepscope_kernel::contracts::{Coupling, PulseCondition, Slope, Sweep};

const CH1: SettingTarget = SettingTarget::Range { channel: 1 };

fn saved(instrument: &Instrument) -> Value {
    serde_json::from_slice(&instrument.to_setup_file().unwrap().to_bytes()).unwrap()
}

fn parsed(value: &Value) -> SetupFile {
    SetupFile::parse(&serde_json::to_vec(value).unwrap()).unwrap()
}

fn parse_error(value: &Value) -> String {
    SetupFile::parse(&serde_json::to_vec(value).unwrap())
        .unwrap_err()
        .message
}

fn state_of(instrument: &Instrument) -> sweepscope_acquisition::InstrumentState {
    instrument.state(1).unwrap()
}

fn customised() -> Instrument {
    let mut instrument = ready_instrument();
    enable_all(&mut instrument);
    instrument.set_channel(2, patch_position(1.5)).unwrap();
    instrument
        .set_channel(
            3,
            ChannelPatch {
                coupling: Some(Coupling::Ac),
                probe: Some(10.0),
                invert: Some(true),
                bandwidth_limit: Some(true),
                ..ChannelPatch::default()
            },
        )
        .unwrap();
    for _ in 0..3 {
        instrument.step(CH1, -1).unwrap();
        instrument.step(SettingTarget::TimeBase, -1).unwrap();
        instrument.step(SettingTarget::TriggerPosition, 1).unwrap();
    }
    instrument.step(SettingTarget::TriggerLevel, 1).unwrap();
    instrument
        .set_trigger(TriggerPatch {
            source: Some(2),
            slope: Some(Slope::Falling),
            sweep: Some(Sweep::Normal),
            ..TriggerPatch::default()
        })
        .unwrap();
    instrument
}

#[test]
fn given_a_ready_instrument_when_saved_then_the_file_has_format_version_and_values_in_volts_and_seconds() {
    let file = saved(&ready_instrument());

    assert_eq!(file["format"], "sweepscope-setup");
    assert_eq!(file["version"], 1);
    assert_eq!(file["channels"][0]["volts_per_div"], 1.0);
    assert_eq!(file["time_base_s_per_div"], 200e-6);
    assert_eq!(file["trigger"]["position_percent"], 50.0);
    assert_eq!(file["trigger"]["source"], 1);
}

#[test]
fn given_a_saved_setup_when_it_is_written_then_it_is_pretty_printed_with_stable_key_order() {
    let bytes = ready_instrument().to_setup_file().unwrap().to_bytes();

    let text = String::from_utf8(bytes).unwrap();

    assert!(text.starts_with("{\n  \"format\": \"sweepscope-setup\",\n  \"version\": 1,\n  \"channels\": ["));
}

#[test]
fn given_no_device_ever_ready_when_the_setup_is_asked_then_there_is_none() {
    let instrument = Instrument::new(Defaults::default());

    assert!(instrument.to_setup_file().is_none());
}

#[test]
fn given_a_saved_setup_and_a_changed_instrument_when_loaded_then_the_instrument_equals_the_saved_one() {
    let original = customised();
    let file = original.to_setup_file().unwrap();
    let mut other = ready_instrument();

    let report = other.apply_setup_file(&file).unwrap();

    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    assert_eq!(state_of(&other), state_of(&original));
    assert_eq!(other.setup(), original.setup());
    assert_eq!(other.to_setup_file(), Some(file));
}

#[test]
fn given_a_range_the_device_does_not_have_when_loaded_then_the_current_range_stays_and_a_warning_names_it() {
    let mut value = saved(&ready_instrument());
    value["channels"][0]["volts_per_div"] = json!(3.0);
    let mut instrument = ready_instrument();

    let report = instrument.apply_setup_file(&parsed(&value)).unwrap();

    assert_eq!(report.warnings, ["CH1 volts/div 3.00 V is not available; kept 1.00 V"]);
    assert_eq!(saved(&instrument)["channels"][0]["volts_per_div"], 1.0);
}

#[test]
fn given_a_time_base_the_device_does_not_have_when_loaded_then_the_current_one_stays_and_a_warning_names_it() {
    let mut value = saved(&ready_instrument());
    value["time_base_s_per_div"] = json!(3.0);
    let mut instrument = ready_instrument();

    let report = instrument.apply_setup_file(&parsed(&value)).unwrap();

    assert_eq!(report.warnings, ["Time/div 3.00 s is not available; kept 200 us"]);
    assert_eq!(saved(&instrument)["time_base_s_per_div"], 200e-6);
}

#[test]
fn given_a_roll_mode_time_base_when_loaded_then_it_is_applied_and_the_instrument_rolls() {
    let mut value = saved(&ready_instrument());
    value["time_base_s_per_div"] = json!(1.0);
    let mut instrument = ready_instrument();

    let report = instrument.apply_setup_file(&parsed(&value)).unwrap();

    assert!(report.warnings.is_empty());
    assert!(instrument.state(1).unwrap().horizontal.roll);
}

#[test]
fn given_a_file_without_pulse_keys_when_loaded_then_the_trigger_is_an_edge() {
    let value = saved(&ready_instrument());
    let mut instrument = ready_instrument();

    instrument.apply_setup_file(&parsed(&value)).unwrap();

    assert!(value["trigger"].get("pulse").is_none());
    assert_eq!(instrument.state(1).unwrap().trigger.kind, TriggerKindName::Edge);
}

#[test]
fn given_a_file_with_pulse_keys_when_loaded_then_the_pulse_trigger_is_restored() {
    let mut value = saved(&ready_instrument());
    value["trigger"]["pulse"] = json!({ "width_s": 0.0002, "condition": "less" });
    let mut instrument = ready_instrument();

    instrument.apply_setup_file(&parsed(&value)).unwrap();

    let state = instrument.state(1).unwrap().trigger;
    assert_eq!(state.kind, TriggerKindName::Pulse);
    let pulse = state.pulse.unwrap();
    assert_eq!(pulse.condition, PulseCondition::Less);
    assert_eq!(pulse.width.text, "200");
}

#[test]
fn given_a_pulse_trigger_when_saved_then_the_keys_are_written_and_load_back_equal() {
    let mut instrument = ready_instrument();
    instrument
        .set_trigger(TriggerPatch {
            kind: Some(TriggerKindName::Pulse),
            width_text: Some("2 ms".into()),
            ..TriggerPatch::default()
        })
        .unwrap();
    let value = saved(&instrument);
    let mut other = ready_instrument();

    other.apply_setup_file(&parsed(&value)).unwrap();

    assert_eq!(value["trigger"]["pulse"]["width_s"], 0.002);
    assert_eq!(
        other.state(1).unwrap().trigger.pulse,
        instrument.state(1).unwrap().trigger.pulse
    );
}

#[test]
fn given_a_pulse_width_out_of_range_when_loaded_then_the_trigger_stays_an_edge_with_a_warning() {
    let mut value = saved(&ready_instrument());
    value["trigger"]["pulse"] = json!({ "width_s": 100.0, "condition": "equal" });
    let mut instrument = ready_instrument();

    let report = instrument.apply_setup_file(&parsed(&value)).unwrap();

    assert_eq!(report.warnings.len(), 1);
    assert_eq!(instrument.state(1).unwrap().trigger.kind, TriggerKindName::Edge);
}

#[test]
fn given_a_probe_the_app_does_not_know_when_loaded_then_the_probe_stays_and_a_warning_names_it() {
    let mut value = saved(&ready_instrument());
    value["channels"][1]["probe"] = json!(20.0);
    let mut instrument = ready_instrument();

    let report = instrument.apply_setup_file(&parsed(&value)).unwrap();

    assert_eq!(report.warnings, ["CH2 probe 20x is not available; kept 1x"]);
}

#[test]
fn given_a_position_beyond_the_hardware_limit_when_loaded_then_it_is_limited_and_a_warning_says_so() {
    let mut value = saved(&ready_instrument());
    value["channels"][0]["position_divs"] = json!(9.0);
    let mut instrument = ready_instrument();

    let report = instrument.apply_setup_file(&parsed(&value)).unwrap();

    assert_eq!(report.warnings.len(), 1);
    assert!(report.warnings[0].starts_with("CH1 position 9.00 div is out of range"));
    assert_eq!(saved(&instrument)["channels"][0]["position_divs"], 4.0);
}

#[test]
fn given_a_trigger_source_that_is_disabled_in_the_file_when_loaded_then_the_first_enabled_channel_is_used() {
    let mut value = saved(&ready_instrument());
    value["trigger"]["source"] = json!(3);
    let mut instrument = ready_instrument();

    let report = instrument.apply_setup_file(&parsed(&value)).unwrap();

    assert_eq!(report.warnings, ["Trigger source CH3 is not enabled; kept CH1"]);
}

#[test]
fn given_no_device_ready_when_a_setup_is_loaded_then_it_is_refused_with_the_connection_reason() {
    let file = ready_instrument().to_setup_file().unwrap();
    let mut instrument = Instrument::new(Defaults::default());

    let error = instrument.apply_setup_file(&file).unwrap_err();

    assert_eq!(error.message, "No scope connected");
}

#[test]
fn given_text_that_is_not_json_when_parsed_then_the_error_says_so() {
    let error = SetupFile::parse(b"{nope").unwrap_err();

    assert!(error.message.contains("not valid JSON"), "{}", error.message);
}

#[test]
fn given_another_format_when_parsed_then_the_error_says_the_format_is_wrong() {
    let mut value = saved(&ready_instrument());
    value["format"] = json!("something-else");

    assert!(parse_error(&value).contains("not a SweepScope setup"));
}

#[test]
fn given_a_newer_version_when_parsed_then_the_error_names_the_version() {
    let mut value = saved(&ready_instrument());
    value["version"] = json!(2);

    assert_eq!(
        parse_error(&value),
        "The setup file was written by a newer SweepScope (version 2)"
    );
}

#[test]
fn given_a_missing_field_when_parsed_then_the_error_names_it() {
    let mut value = saved(&ready_instrument());
    value.as_object_mut().unwrap().remove("trigger");

    assert!(parse_error(&value).contains("trigger"));
}

#[test]
fn given_a_file_that_enables_no_channel_when_parsed_then_it_is_refused() {
    let mut value = saved(&ready_instrument());
    value["channels"][0]["enabled"] = json!(false);

    assert_eq!(parse_error(&value), "The setup file enables no channel");
}

#[test]
fn given_a_trigger_source_outside_one_to_four_when_parsed_then_it_is_refused() {
    let mut value = saved(&ready_instrument());
    value["trigger"]["source"] = json!(5);

    assert!(parse_error(&value).contains("trigger source"));
}

#[test]
fn given_a_held_setup_when_the_first_device_is_ready_then_the_instrument_holds_it_and_stays_stopped() {
    let original = customised();
    let mut instrument = Instrument::new(Defaults::default());
    instrument.hold_setup(original.to_setup_file().unwrap());

    let restore = instrument.on_ready(capabilities());

    assert!(!restore.resume);
    assert_eq!(state_of(&instrument).channels, state_of(&original).channels);
    assert!(state_of(&instrument).setup_notice.is_none());
}

#[test]
fn given_a_held_setup_with_an_unavailable_value_when_ready_then_a_notice_names_it() {
    let mut value = saved(&ready_instrument());
    value["channels"][0]["volts_per_div"] = json!(3.0);
    let mut instrument = Instrument::new(Defaults::default());
    instrument.hold_setup(parsed(&value));

    instrument.on_ready(capabilities());

    let notice = state_of(&instrument).setup_notice.unwrap();
    assert_eq!(notice.id, 1);
    assert!(
        notice.text.contains("CH1 volts/div 3.00 V is not available"),
        "{}",
        notice.text
    );
}

#[test]
fn given_a_new_notice_when_raised_again_then_it_gets_a_new_id() {
    let mut instrument = ready_instrument();

    instrument.set_notice("first".into());
    instrument.set_notice("second".into());

    let notice = state_of(&instrument).setup_notice.unwrap();
    assert_eq!((notice.id, notice.text.as_str()), (2, "second"));
}
