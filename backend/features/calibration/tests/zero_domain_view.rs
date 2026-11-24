mod common;

use common::RANGES;
use sweepscope_calibration::domain::offsets::StoredUnit;
use sweepscope_calibration::domain::texts;
use sweepscope_calibration::domain::view::{unit_calibration, Slice};

fn stored(volts: [Vec<f64>; 4]) -> StoredUnit {
    StoredUnit {
        stored_at_utc: "2026-11-24T10:00:00Z".to_owned(),
        volts,
    }
}

fn row_values(unit: &sweepscope_calibration::UnitCalibration) -> Vec<(String, String)> {
    unit.tooltip
        .rows
        .iter()
        .map(|row| (row.label.clone(), row.value.clone()))
        .collect()
}

#[test]
fn given_no_stored_unit_when_the_tooltip_is_built_then_it_shows_the_typical_errors_golden() {
    let unit = unit_calibration(None, &RANGES);

    assert!(!unit.calibrated);
    assert_eq!(unit.stored_at_utc, None);
    assert_eq!(
        row_values(&unit),
        [
            ("2 mV/div", "+0.66 mV (8 counts)"),
            ("5 mV/div", "+1.04 mV (5 counts)"),
            ("10 mV/div", "+0.83 mV (2 counts)"),
            ("20 mV/div", "+0.83 mV (1 count)"),
            ("50 mV/div and above", "up to 2 counts between channels"),
        ]
        .map(|(a, b)| (a.to_owned(), b.to_owned()))
    );
    assert_eq!(
        unit.tooltip.notes,
        [
            "Typical zero error of one measured unit:",
            "Gain: about 2 % from 50 mV to 1 V/div (one unit measured), not verified outside that range.",
            "At extreme vertical positions readings can still be off by 2 to 3 counts (about 0.1 division).",
            "Resolution is one count, 1/24 of a division.",
        ]
    );
}

#[test]
fn given_a_stored_unit_when_the_tooltip_is_built_then_each_channel_shows_its_largest_offset_relative_to_the_range() {
    let mut ch1 = vec![0.0; 12];
    ch1[0] = 0.000_66;
    ch1[11] = 0.01;
    let mut ch2 = vec![0.0; 12];
    ch2[1] = -0.001_04;
    ch2[4] = 0.000_2;
    let unit = unit_calibration(Some(&stored([ch1, ch2, vec![0.0; 12], vec![]])), &RANGES);

    assert!(unit.calibrated);
    assert_eq!(unit.stored_at_utc.as_deref(), Some("2026-11-24T10:00:00Z"));
    let rows = row_values(&unit);
    assert_eq!(rows[0], ("CH1".to_owned(), "+0.66 mV at 2 mV/div".to_owned()));
    assert_eq!(rows[1], ("CH2".to_owned(), "\u{2212}1.04 mV at 5 mV/div".to_owned()));
    assert_eq!(rows[2], ("CH3".to_owned(), "+0.00 mV at 2 mV/div".to_owned()));
    assert_eq!(rows[3], ("CH4".to_owned(), "-".to_owned()));
}

#[test]
fn given_a_stored_unit_when_the_notes_are_built_then_they_start_with_what_was_removed() {
    let unit = unit_calibration(Some(&stored(Default::default())), &RANGES);

    assert_eq!(
        unit.tooltip.notes[0],
        "Zero error removed per channel and range, measured with the input shorted (about 12 000 samples averaged)."
    );
    assert_eq!(unit.tooltip.notes.len(), 4);
}

#[test]
fn given_an_unchanged_input_when_the_slice_is_updated_then_nothing_is_announced_and_the_revision_stays() {
    let mut slice = Slice::new(texts::unavailable(texts::NO_DEVICE));

    let announced = slice.update(texts::unavailable(texts::NO_DEVICE), None, None);

    assert_eq!(announced, None);
    assert_eq!(slice.state().revision, 0);
}

#[test]
fn given_a_changed_availability_when_the_slice_is_updated_then_the_revision_moves_and_the_state_is_announced() {
    let mut slice = Slice::new(texts::unavailable(texts::NO_DEVICE));

    let announced = slice
        .update(texts::available(), Some(unit_calibration(None, &RANGES)), None)
        .unwrap();

    assert_eq!(announced.revision, 1);
    assert!(announced.availability.enabled);
    assert_eq!(slice.state(), announced);
}

#[test]
fn given_the_reasons_when_listed_then_they_match_the_spec_texts() {
    assert_eq!(texts::NO_DEVICE, "No device");
    assert_eq!(texts::NEEDS_READY_SCOPE, "Zero calibration needs a ready scope");
    assert_eq!(texts::NO_SERIAL, "This scope reports no serial number");
    assert_eq!(texts::AUTO_SETUP_RUNNING, "Auto-setup is running");
    assert_eq!(
        texts::unavailable(texts::NO_SERIAL).reason.as_deref(),
        Some(texts::NO_SERIAL)
    );
}
