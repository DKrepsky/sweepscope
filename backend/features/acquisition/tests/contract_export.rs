use std::path::Path;

use sweepscope_acquisition::{
    AcquisitionStatus, Availability, ChannelPatch, ChannelState, ControlsAvailability, FormattedValue, InstrumentFacts,
    InstrumentState, Phase, ScaleChoice, SettingTarget, SetupLoaded, SetupNotice, TriggerPatch,
};
use sweepscope_kernel::contracts::{Slope, Sweep};
use ts_rs::{Config, TS};

fn generated(name: &str) -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../frontend/src/api/generated")
        .join(name);
    std::fs::read_to_string(file).unwrap()
}

#[test]
fn given_the_state_slices_when_exported_then_revisions_are_numbers_and_fields_are_camel_case() {
    let config = Config::from_env();
    InstrumentState::export_all(&config).unwrap();
    AcquisitionStatus::export_all(&config).unwrap();
    InstrumentFacts::export_all(&config).unwrap();

    assert!(generated("InstrumentState.ts").contains("revision: number"));
    assert!(generated("InstrumentState.ts").contains("channels: Array<ChannelState>"));
    assert!(generated("AcquisitionStatus.ts").contains("phase: Phase"));
    assert!(generated("InstrumentFacts.ts").contains("sampleRate: FormattedValue | null"));
}

#[test]
fn given_the_channel_and_scale_types_when_exported_then_the_ui_fields_exist() {
    let config = Config::from_env();
    ChannelState::export_all(&config).unwrap();
    ScaleChoice::export_all(&config).unwrap();
    FormattedValue::export_all(&config).unwrap();
    Availability::export_all(&config).unwrap();
    Phase::export_all(&config).unwrap();

    let channel = generated("ChannelState.ts");
    for field in [
        "maxPositionDivs: number",
        "bandwidthLimit: boolean",
        "probeText: string",
        "enable: Availability",
        "coupling: Coupling",
    ] {
        assert!(channel.contains(field), "{field} missing in {channel}");
    }
    assert!(!channel.contains("unverified"), "{channel}");
    assert!(!generated("ScaleChoice.ts").contains("verified"));
    assert!(generated("ScaleChoice.ts").contains("faster: Availability"));
    assert!(generated("FormattedValue.ts").contains("unit: string"));
    assert!(generated("Availability.ts").contains("reason: string | null"));
    assert!(generated("Phase.ts").contains("\"waitingForTrigger\""));
}

#[test]
fn given_the_setup_types_when_exported_then_the_notice_and_the_load_report_have_the_documented_fields() {
    let config = Config::from_env();
    SetupNotice::export_all(&config).unwrap();
    SetupLoaded::export_all(&config).unwrap();
    InstrumentState::export_all(&config).unwrap();
    ControlsAvailability::export_all(&config).unwrap();

    assert!(generated("SetupNotice.ts").contains("id: number"));
    assert!(generated("SetupLoaded.ts").contains("warnings: Array<string>"));
    assert!(generated("InstrumentState.ts").contains("setupNotice: SetupNotice | null"));
    assert!(generated("ControlsAvailability.ts").contains("files: Availability"));
}

#[test]
fn given_the_patch_types_when_exported_then_every_field_is_optional() {
    let config = Config::from_env();
    ChannelPatch::export_all(&config).unwrap();
    TriggerPatch::export_all(&config).unwrap();

    let channel = generated("ChannelPatch.ts");
    for field in [
        "enabled?: boolean",
        "positionDivs?: number",
        "coupling?: Coupling",
        "bandwidthLimit?: boolean",
        "probe?: number",
        "invert?: boolean",
    ] {
        assert!(channel.contains(field), "{field} missing in {channel}");
    }
    let trigger = generated("TriggerPatch.ts");
    assert!(
        trigger.contains("source?: number") && trigger.contains("slope?: Slope") && trigger.contains("sweep?: Sweep")
    );
}

#[test]
fn given_the_setting_target_when_exported_then_it_is_a_union_tagged_by_kind() {
    SettingTarget::export_all(&Config::from_env()).unwrap();

    let target = generated("SettingTarget.ts");
    assert!(target.contains("\"kind\": \"range\", channel: number"), "{target}");
    assert!(target.contains("\"kind\": \"timeBase\""), "{target}");
    assert!(target.contains("\"kind\": \"triggerPosition\""), "{target}");
    assert!(target.contains("\"kind\": \"triggerLevel\""), "{target}");
}

#[test]
fn given_a_json_patch_with_one_field_when_deserialized_then_the_others_are_none() {
    let patch: ChannelPatch = serde_json::from_str(r#"{"positionDivs": -1.5}"#).unwrap();

    assert_eq!(
        patch,
        ChannelPatch {
            position_divs: Some(-1.5),
            ..ChannelPatch::default()
        }
    );
}

#[test]
fn given_a_json_trigger_patch_when_deserialized_then_enums_use_their_camel_case_names() {
    let patch: TriggerPatch = serde_json::from_str(r#"{"slope": "falling", "sweep": "normal"}"#).unwrap();

    assert_eq!(patch.slope, Some(Slope::Falling));
    assert_eq!(patch.sweep, Some(Sweep::Normal));
    assert_eq!(patch.source, None);
}

#[test]
fn given_json_targets_when_deserialized_then_the_kind_selects_the_variant() {
    let range: SettingTarget = serde_json::from_str(r#"{"kind": "range", "channel": 2}"#).unwrap();
    let time_base: SettingTarget = serde_json::from_str(r#"{"kind": "timeBase"}"#).unwrap();

    assert_eq!(range, SettingTarget::Range { channel: 2 });
    assert_eq!(time_base, SettingTarget::TimeBase);
}
