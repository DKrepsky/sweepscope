use engineering_format::{parse_seconds, ParseError};

fn assert_parses(text: &str, seconds: f64) {
    let got = parse_seconds(text).unwrap_or_else(|e| panic!("{text:?}: {e:?}"));
    assert!((got - seconds).abs() <= seconds * 1e-12, "{text:?} gave {got}");
}

#[test]
fn given_valid_texts_when_parsed_then_the_value_is_in_seconds() {
    let table = [
        ("500 us", 500e-6),
        ("500us", 500e-6),
        ("500\u{b5}s", 500e-6),
        ("500 \u{b5}s", 500e-6),
        ("500 \u{3bc}s", 500e-6),
        ("0.5 ms", 0.5e-3),
        ("2e-3", 2e-3),
        ("1.5 s", 1.5),
        ("10 ns", 10e-9),
        ("  3 ms  ", 3e-3),
    ];
    for (text, seconds) in table {
        assert_parses(text, seconds);
    }
}

#[test]
fn given_an_empty_text_when_parsed_then_it_is_empty() {
    assert_eq!(parse_seconds(""), Err(ParseError::Empty));
    assert_eq!(parse_seconds("   "), Err(ParseError::Empty));
}

#[test]
fn given_text_that_is_not_a_duration_when_parsed_then_it_is_not_a_number() {
    for text in ["abc", "5 m", "NaN", "inf", "infs", "ms", "1 2 ms", "5 hz"] {
        assert_eq!(parse_seconds(text), Err(ParseError::NotANumber), "{text:?}");
    }
}

#[test]
fn given_a_negative_duration_when_parsed_then_it_is_refused() {
    assert_eq!(parse_seconds("-1 ms"), Err(ParseError::Negative));
}
