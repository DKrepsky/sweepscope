mod common;

use std::collections::BTreeSet;

use common::*;
use sweepscope_device::{Capabilities, ScopeDevice};

fn page() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../docs/accuracy.md");
    std::fs::read_to_string(path).expect("docs/accuracy.md exists")
}

fn capabilities() -> Capabilities {
    rig().device.capabilities().clone()
}

fn range_label(volts_per_div: f64) -> String {
    let millivolts = (volts_per_div * 1000.0).round();
    if millivolts < 1000.0 {
        format!("{millivolts} mV/div")
    } else {
        format!("{} V/div", volts_per_div.round())
    }
}

fn table_rows(page: &str, heading: &str) -> Vec<Vec<String>> {
    let section = page
        .split("\n## ")
        .find(|part| part.starts_with(heading))
        .expect("section exists");
    section
        .lines()
        .filter(|line| line.starts_with('|') && !line.starts_with("|---"))
        .skip(1)
        .map(|line| {
            line.trim_matches('|')
                .split('|')
                .map(|cell| cell.trim().to_string())
                .collect()
        })
        .collect()
}

fn measured_time_base_indexes(page: &str) -> BTreeSet<usize> {
    let line = page
        .lines()
        .find_map(|line| line.split("Measured indexes: ").nth(1))
        .expect("measured indexes line");
    let list = line.split_once('.').map_or(line, |(list, _)| list);
    list.split(", ")
        .flat_map(|part| match part.split_once(" to ") {
            Some((from, to)) => from.parse::<usize>().unwrap()..=to.parse::<usize>().unwrap(),
            None => part.parse::<usize>().unwrap()..=part.parse::<usize>().unwrap(),
        })
        .collect()
}

#[test]
fn given_the_capability_table_when_the_vertical_ranges_page_is_read_then_it_lists_every_range_in_order() {
    let caps = capabilities();

    let listed: Vec<String> = table_rows(&page(), "Vertical ranges")
        .into_iter()
        .map(|row| row[0].clone())
        .collect();

    let expected: Vec<String> = caps
        .vertical_ranges
        .iter()
        .map(|range| range_label(range.volts_per_div))
        .collect();
    assert_eq!(listed, expected);
}

#[test]
fn given_the_capability_table_when_the_page_status_is_read_then_verified_ranges_match_the_table() {
    let caps = capabilities();

    let rows = table_rows(&page(), "Vertical ranges");

    for (range, row) in caps.vertical_ranges.iter().zip(rows) {
        let page_says_verified = row[1] == "Verified";
        assert_eq!(page_says_verified, range.verified, "{}", row[0]);
    }
}

#[test]
fn given_the_capability_table_when_the_time_base_line_is_read_then_measured_indexes_match_the_table() {
    let caps = capabilities();

    let measured = measured_time_base_indexes(&page());

    let expected: BTreeSet<usize> = caps
        .time_bases
        .iter()
        .enumerate()
        .filter(|(_, step)| step.verified)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(measured, expected);
}

#[test]
fn given_the_capability_table_when_the_time_base_count_is_read_then_the_page_states_the_same_count() {
    let caps = capabilities();

    let page = page();

    assert!(page.contains(&format!("There are {} time bases", caps.time_bases.len())));
}

#[test]
fn given_the_page_when_its_wording_is_read_then_it_states_no_tolerance_and_uses_about() {
    let page = page().to_lowercase();

    assert!(page.contains("not tolerances"));
    assert!(page.contains("about 0.1 division"));
    assert!(!page.contains('±'));
}
