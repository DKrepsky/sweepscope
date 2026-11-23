//! Reads `golden_measurement.tsv`, vectors for the four-digit measured quantities (Rust only).
use engineering_format::{format, Quantity};

const GOLDEN: &str = include_str!("golden_measurement.tsv");

fn quantity(name: &str) -> Quantity {
    match name {
        "frequency" => Quantity::Frequency,
        "period" => Quantity::Period,
        other => panic!("unknown quantity {other}"),
    }
}

#[test]
fn given_every_measurement_row_when_formatted_then_text_and_unit_match() {
    let rows: Vec<&str> = GOLDEN.lines().filter(|line| !line.is_empty()).collect();
    assert!(!rows.is_empty());
    for row in rows {
        let [value, name, text, unit] = row.split('\t').collect::<Vec<_>>()[..] else {
            panic!("row needs four columns: {row}");
        };
        let shown = format(value.parse().expect("value"), quantity(name));
        assert_eq!((shown.text.as_str(), shown.unit.as_str()), (text, unit), "{row}");
    }
}
