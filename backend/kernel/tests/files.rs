use std::path::PathBuf;

use sweepscope_kernel::testing::ScriptedPicker;
use sweepscope_kernel::{FileFilter, FilePicker};

fn filter() -> FileFilter {
    FileFilter {
        label: "CSV".to_owned(),
        extensions: vec!["csv".to_owned()],
    }
}

#[test]
fn given_a_scripted_picker_when_asked_to_save_twice_then_answers_come_in_order_and_names_are_recorded() {
    let picker = ScriptedPicker::new(vec![Some(PathBuf::from("/x/a.csv")), None], vec![]);

    let first = picker.pick_save("one.csv", &filter()).unwrap();
    let second = picker.pick_save("two.csv", &filter()).unwrap();

    assert_eq!(first, Some(PathBuf::from("/x/a.csv")));
    assert_eq!(second, None);
    assert_eq!(picker.suggested_names(), ["one.csv", "two.csv"]);
}

#[test]
fn given_a_scripted_picker_when_asked_to_open_then_the_open_queue_answers() {
    let picker = ScriptedPicker::new(vec![], vec![Some(PathBuf::from("/x/setup.json"))]);

    assert_eq!(
        picker.pick_open(&filter()).unwrap(),
        Some(PathBuf::from("/x/setup.json"))
    );
}

#[test]
fn given_an_empty_queue_when_a_dialog_is_requested_then_it_fails_as_an_unscripted_dialog() {
    let picker = ScriptedPicker::default();

    let error = picker.pick_open(&filter()).unwrap_err();

    assert!(error.message.contains("no scripted answer"), "{}", error.message);
}
