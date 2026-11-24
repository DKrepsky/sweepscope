use std::path::Path;

use sweepscope_acquisition::{
    AcquisitionStatus, AutosetOutcomeKind, ControlsAvailability, HorizontalState, InstrumentState, MathOperation,
    MathPatch, MathState, MathStatus, Phase, PulseState, SettingTarget, TriggerKindName, TriggerPatch, TriggerState,
};
use sweepscope_kernel::contracts::PulseCondition;
use ts_rs::{Config, TS};

fn generated(name: &str) -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../frontend/src/api/generated")
        .join(name);
    std::fs::read_to_string(file).unwrap()
}

#[test]
fn given_the_analyse_state_when_exported_then_roll_math_pulse_and_autoset_fields_exist() {
    let config = Config::from_env();
    InstrumentState::export_all(&config).unwrap();
    HorizontalState::export_all(&config).unwrap();
    TriggerState::export_all(&config).unwrap();
    PulseState::export_all(&config).unwrap();
    ControlsAvailability::export_all(&config).unwrap();
    AcquisitionStatus::export_all(&config).unwrap();
    AutosetOutcomeKind::export_all(&config).unwrap();
    Phase::export_all(&config).unwrap();

    assert!(generated("InstrumentState.ts").contains("math: MathState"));
    assert!(generated("HorizontalState.ts").contains("roll: boolean"));
    assert!(generated("TriggerState.ts").contains("pulse: PulseState | null"));
    assert!(generated("PulseState.ts").contains("verified: boolean"));
    for field in [
        "single: Availability",
        "autoset: Availability",
        "record: Availability",
        "frameViews: Availability",
    ] {
        assert!(generated("ControlsAvailability.ts").contains(field), "{field}");
    }
    assert!(generated("AcquisitionStatus.ts").contains("autoset: AutosetStatus"));
    assert!(generated("AutosetOutcomeKind.ts").contains("\"done\" | \"noSignal\" | \"failed\""));
    assert!(generated("Phase.ts").contains("\"done\""));
}

#[test]
fn given_the_math_types_when_exported_then_the_status_is_a_union_tagged_by_kind() {
    let config = Config::from_env();
    MathState::export_all(&config).unwrap();
    MathStatus::export_all(&config).unwrap();
    MathOperation::export_all(&config).unwrap();
    MathPatch::export_all(&config).unwrap();

    assert!(generated("MathStatus.ts").contains("\"kind\": \"unavailable\", reason: string"));
    assert!(generated("MathOperation.ts").contains("\"invertA\""));
    assert!(generated("MathState.ts").contains("sourceB: number | null"));
    assert!(generated("MathPatch.ts").contains("positionDivs?: number"));
}

#[test]
fn given_a_json_pulse_patch_when_deserialized_then_kind_condition_and_width_text_are_read() {
    let patch: TriggerPatch =
        serde_json::from_str(r#"{"kind": "pulse", "condition": "greater", "widthText": "500 us"}"#).unwrap();

    assert_eq!(patch.kind, Some(TriggerKindName::Pulse));
    assert_eq!(patch.condition, Some(PulseCondition::Greater));
    assert_eq!(patch.width_text.as_deref(), Some("500 us"));
}

#[test]
fn given_the_math_setting_targets_when_deserialized_then_the_kind_selects_the_variant() {
    let scale: SettingTarget = serde_json::from_str(r#"{"kind": "mathScale"}"#).unwrap();
    let position: SettingTarget = serde_json::from_str(r#"{"kind": "mathPosition"}"#).unwrap();

    assert_eq!(scale, SettingTarget::MathScale);
    assert_eq!(position, SettingTarget::MathPosition);
}
