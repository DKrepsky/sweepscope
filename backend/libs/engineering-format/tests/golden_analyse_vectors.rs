//! Reads `golden_analyse.tsv`, vectors for the M3 quantities (the TypeScript formatter reads the same file).
use engineering_format::{format, Quantity};

const GOLDEN: &str = include_str!("golden_analyse.tsv");

fn quantity(name: &str) -> Quantity {
    match name {
        "decibel" => Quantity::Decibel,
        "volts_squared_per_div" => Quantity::VoltsSquaredPerDiv,
        "frequency" => Quantity::Frequency,
        other => panic!("unknown quantity {other}"),
    }
}

#[test]
fn given_every_analyse_row_when_formatted_then_text_and_unit_match() {
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
