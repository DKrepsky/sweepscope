use std::path::Path;

use sweepscope_display::Persistence;
use ts_rs::{Config, TS};

fn generated(name: &str) -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../frontend/src/api/generated")
        .join(name);
    std::fs::read_to_string(file).unwrap()
}

#[test]
fn given_the_persistence_values_when_exported_then_they_are_the_wire_strings() {
    Persistence::export_all(&Config::from_env()).unwrap();

    let text = generated("Persistence.ts");
    for value in ["\"off\"", "\"500ms\"", "\"1s\"", "\"2s\"", "\"infinite\""] {
        assert!(text.contains(value), "{value} missing in {text}");
    }
}

#[test]
fn given_json_persistence_values_when_deserialized_then_each_maps_to_its_variant() {
    let parse = |text: &str| serde_json::from_str::<Persistence>(text).unwrap();

    assert_eq!(parse(r#""off""#), Persistence::Off);
    assert_eq!(parse(r#""500ms""#), Persistence::Ms500);
    assert_eq!(parse(r#""1s""#), Persistence::S1);
    assert_eq!(parse(r#""2s""#), Persistence::S2);
    assert_eq!(parse(r#""infinite""#), Persistence::Infinite);
}
