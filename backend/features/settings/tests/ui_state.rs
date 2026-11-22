use std::sync::Arc;

use sweepscope_kernel::testing::MemoryStorage;
use sweepscope_kernel::{Bus, ErrorCategory, Storage};
use sweepscope_settings::{LoadUiState, SaveUiState, SettingsDeps, SettingsModule, UiStateEntry};
use ts_rs::{Config, TS};

fn module_on(storage: &MemoryStorage) -> Bus {
    let bus = Bus::new();
    let deps = SettingsDeps {
        bus: bus.clone(),
        storage: Arc::new(storage.clone()),
    };
    SettingsModule::new(deps).unwrap();
    bus
}

fn entry(namespace: &str, json: &str) -> UiStateEntry {
    UiStateEntry {
        namespace: namespace.into(),
        version: 1,
        json: json.into(),
    }
}

fn load(bus: &Bus, namespace: &str) -> Result<Option<UiStateEntry>, sweepscope_kernel::KernelError> {
    bus.request(LoadUiState {
        namespace: namespace.into(),
    })
}

#[test]
fn given_no_file_when_ui_state_is_loaded_then_none_is_returned() {
    let bus = module_on(&MemoryStorage::default());

    assert_eq!(load(&bus, "app.text-scale"), Ok(None));
}

#[test]
fn given_a_saved_entry_when_loaded_by_a_new_module_instance_then_it_is_equal() {
    let storage = MemoryStorage::default();
    let saved = entry("app.text-scale", "{\"percent\":125}");
    module_on(&storage).request(SaveUiState(saved.clone())).unwrap();

    let reloaded = load(&module_on(&storage), "app.text-scale");

    assert_eq!(reloaded, Ok(Some(saved)));
}

#[test]
fn given_two_namespaces_when_one_is_saved_again_then_the_other_is_untouched() {
    let bus = module_on(&MemoryStorage::default());
    bus.request(SaveUiState(entry("a", "1"))).unwrap();
    bus.request(SaveUiState(entry("b", "2"))).unwrap();

    bus.request(SaveUiState(entry("a", "3"))).unwrap();

    assert_eq!(load(&bus, "b"), Ok(Some(entry("b", "2"))));
    assert_eq!(load(&bus, "a"), Ok(Some(entry("a", "3"))));
}

#[test]
fn given_corrupt_json_when_loaded_then_a_recoverable_error_returns_and_the_file_stays() {
    let storage = MemoryStorage::default();
    storage.write("settings.json", b"{ not json").unwrap();

    let error = load(&module_on(&storage), "a").unwrap_err();

    assert_eq!(error.category, ErrorCategory::Recoverable);
    assert_eq!(storage.read("settings.json"), Ok(Some(b"{ not json".to_vec())));
}

#[test]
fn given_corrupt_json_when_saving_then_the_old_bytes_are_kept_as_a_backup_and_the_save_succeeds() {
    let storage = MemoryStorage::default();
    storage.write("settings.json", b"{ not json").unwrap();
    let bus = module_on(&storage);

    bus.request(SaveUiState(entry("a", "1"))).unwrap();

    assert_eq!(storage.read("settings.json.corrupt"), Ok(Some(b"{ not json".to_vec())));
    assert_eq!(load(&bus, "a"), Ok(Some(entry("a", "1"))));
}

#[test]
fn given_a_file_of_a_newer_version_when_saving_then_a_recoverable_error_returns_and_the_file_is_unchanged() {
    let storage = MemoryStorage::default();
    let future = br#"{"version":2,"ui":{},"extra":true}"#;
    storage.write("settings.json", future).unwrap();

    let error = module_on(&storage).request(SaveUiState(entry("a", "1"))).unwrap_err();

    assert_eq!(error.category, ErrorCategory::Recoverable);
    assert_eq!(storage.read("settings.json"), Ok(Some(future.to_vec())));
}

#[test]
fn given_a_second_module_on_the_same_bus_when_created_then_registration_fails() {
    let storage = MemoryStorage::default();
    let bus = module_on(&storage);

    let deps = SettingsDeps {
        bus,
        storage: Arc::new(storage),
    };

    assert!(SettingsModule::new(deps).is_err());
}

#[test]
fn given_the_ui_state_entry_when_exported_then_the_generated_file_lists_its_fields() {
    UiStateEntry::export_all(&Config::from_env()).unwrap();

    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../frontend/src/api/generated/UiStateEntry.ts"
    );
    let text = std::fs::read_to_string(path).unwrap();
    assert!(text.contains("namespace: string") && text.contains("version: number") && text.contains("json: string"));
}
