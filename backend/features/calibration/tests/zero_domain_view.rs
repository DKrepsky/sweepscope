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

const TYPICAL: [[&str; 3]; 12] = [
    ["+0.66 mV at 2 mV/div", "+6.6 mV at 20 mV/div", "+66 mV at 200 mV/div"],
    ["+1.04 mV at 5 mV/div", "+10.4 mV at 50 mV/div", "+104 mV at 500 mV/div"],
    ["+0.83 mV at 10 mV/div", "+8.3 mV at 100 mV/div", "+83 mV at 1 V/div"],
    ["+0.83 mV at 20 mV/div", "+8.3 mV at 200 mV/div", "+83 mV at 2 V/div"],
    [
        "up to 4.14 mV at 50 mV/div",
        "up to 41.4 mV at 500 mV/div",
        "up to 414 mV at 5 V/div",
    ],
    [
        "up to 8.28 mV at 100 mV/div",
        "up to 82.8 mV at 1 V/div",
        "up to 828 mV at 10 V/div",
    ],
    [
        "up to 16.6 mV at 200 mV/div",
        "up to 166 mV at 2 V/div",
        "up to 1.66 V at 20 V/div",
    ],
    [
        "up to 41.4 mV at 500 mV/div",
        "up to 414 mV at 5 V/div",
        "up to 4.14 V at 50 V/div",
    ],
    [
        "up to 82.8 mV at 1 V/div",
        "up to 828 mV at 10 V/div",
        "up to 8.28 V at 100 V/div",
    ],
    [
        "up to 166 mV at 2 V/div",
        "up to 1.66 V at 20 V/div",
        "up to 16.6 V at 200 V/div",
    ],
    [
        "up to 414 mV at 5 V/div",
        "up to 4.14 V at 50 V/div",
        "up to 41.4 V at 500 V/div",
    ],
    [
        "up to 828 mV at 10 V/div",
        "up to 8.28 V at 100 V/div",
        "up to 82.8 V at 1 kV/div",
    ],
];

fn cells(unit: &sweepscope_calibration::UnitCalibration, channel: usize) -> Vec<[String; 3]> {
    unit.tooltip.channels[channel].cells.clone()
}

fn texts_of(rows: [[&str; 3]; 12]) -> Vec<[String; 3]> {
    rows.iter().map(|row| row.map(str::to_owned)).collect()
}

#[test]
fn given_no_stored_unit_when_the_tooltip_is_built_then_every_channel_shows_the_typical_error_of_all_ranges_and_probes_golden(
) {
    let unit = unit_calibration(None, &RANGES);

    assert!(!unit.calibrated);
    assert_eq!(unit.stored_at_utc, None);
    assert_eq!(unit.tooltip.heading, "Typical zero error of one measured unit:");
    for channel in 0..4 {
        assert_eq!(cells(&unit, channel), texts_of(TYPICAL), "channel {channel}");
    }
}

fn calibrated_unit() -> sweepscope_calibration::UnitCalibration {
    let mut ch1 = vec![0.0; 12];
    ch1[0] = 0.000_66;
    ch1[11] = 0.01;
    let mut ch2 = vec![0.0; 12];
    ch2[1] = -0.001_04;
    ch2[4] = 0.000_2;
    unit_calibration(Some(&stored([ch1, ch2, vec![0.0; 12], vec![0.001]])), &RANGES)
}

#[test]
fn given_a_stored_unit_when_the_tooltip_is_built_then_the_heading_says_what_was_removed_and_the_date_is_kept() {
    let unit = calibrated_unit();

    assert!(unit.calibrated);
    assert_eq!(unit.stored_at_utc.as_deref(), Some("2026-11-24T10:00:00Z"));
    assert_eq!(
        unit.tooltip.heading,
        "Zero error removed per channel and range, measured with the input shorted (about 12 000 samples averaged)."
    );
}

#[test]
fn given_a_stored_channel_when_the_tooltip_is_built_then_each_range_shows_its_offset_for_the_three_probes_golden() {
    let unit = calibrated_unit();

    let ch1 = cells(&unit, 0);

    assert_eq!(ch1.len(), 12);
    assert_eq!(
        ch1[0],
        ["+0.66 mV at 2 mV/div", "+6.6 mV at 20 mV/div", "+66 mV at 200 mV/div"]
    );
    assert_eq!(ch1[1], ["0 V at 5 mV/div", "0 V at 50 mV/div", "0 V at 500 mV/div"]);
    assert_eq!(
        ch1[11],
        ["+10 mV at 10 V/div", "+100 mV at 100 V/div", "+1000 mV at 1 kV/div"]
    );
}

#[test]
fn given_a_negative_stored_offset_when_the_tooltip_is_built_then_it_has_a_true_minus() {
    let unit = calibrated_unit();

    assert_eq!(
        cells(&unit, 1)[1],
        [
            "\u{2212}1.04 mV at 5 mV/div",
            "\u{2212}10.4 mV at 50 mV/div",
            "\u{2212}104 mV at 500 mV/div"
        ]
    );
}

#[test]
fn given_a_channel_without_stored_offsets_when_the_tooltip_is_built_then_every_cell_is_a_dash() {
    let unit = unit_calibration(
        Some(&stored([vec![0.0; 12], vec![0.0; 12], vec![0.0; 12], vec![0.001]])),
        &RANGES,
    );

    let ch4 = cells(&unit, 3);

    assert_eq!(ch4.len(), 12);
    assert_eq!(
        ch4[0],
        ["+1 mV at 2 mV/div", "+10 mV at 20 mV/div", "+100 mV at 200 mV/div"]
    );
    assert!(ch4[1..].iter().all(|cell| cell.iter().all(|text| text == "-")));
}

#[test]
fn given_a_slice_range_index_when_the_cell_is_read_then_it_matches_the_capability_range_at_that_index() {
    let unit = unit_calibration(None, &RANGES);

    for (index, cell) in cells(&unit, 0).iter().enumerate() {
        let at_probe_one = cell[0].rsplit(" at ").next().unwrap();
        let range = RANGES[index];
        let expected = if range < 1.0 {
            format!("{} mV/div", (range * 1e3).round())
        } else {
            format!("{} V/div", range.round())
        };
        assert_eq!(at_probe_one, expected, "range index {index}");
    }
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
