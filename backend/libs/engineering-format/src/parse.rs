/// Longer suffixes first, so `ms` is not read as `s`.
const UNITS: [(&str, f64); 6] = [
    ("ms", 1e-3),
    ("us", 1e-6),
    ("\u{b5}s", 1e-6),
    ("\u{3bc}s", 1e-6),
    ("ns", 1e-9),
    ("s", 1.0),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    NotANumber,
    Negative,
}

/// Parses a duration such as `500 us`, `0.5ms`, `2e-3` or `1.5 s`; a bare number is seconds.
pub fn parse_seconds(text: &str) -> Result<f64, ParseError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(ParseError::Empty);
    }
    let (number, factor) = split_unit(text);
    let value: f64 = number.trim().parse().map_err(|_| ParseError::NotANumber)?;
    if !value.is_finite() {
        return Err(ParseError::NotANumber);
    }
    if value < 0.0 {
        return Err(ParseError::Negative);
    }
    Ok(value * factor)
}

fn split_unit(text: &str) -> (&str, f64) {
    UNITS
        .iter()
        .find_map(|(suffix, factor)| text.strip_suffix(suffix).map(|number| (number, *factor)))
        .unwrap_or((text, 1.0))
}
