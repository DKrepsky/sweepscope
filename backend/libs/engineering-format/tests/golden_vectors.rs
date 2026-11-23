//! Reads `golden.tsv`, the vectors the TypeScript port in `frontend/src/app/formatting` is checked against.
use engineering_format::{format, Quantity};

const GOLDEN: &str = include_str!("golden.tsv");

fn quantity(name: &str) -> Quantity {
    match name {
        "volts" => Quantity::Volts,
        "seconds" => Quantity::Seconds,
        "hertz" => Quantity::Hertz,
        other => panic!("unknown quantity {other}"),
    }
}

#[test]
fn given_every_golden_row_when_formatted_then_text_and_unit_match() {
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
