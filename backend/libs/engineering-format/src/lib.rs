//! Engineering notation for instrument readouts: SI prefix, three significant digits, a unit per quantity.
#![forbid(unsafe_code)]

mod quantity;
mod scale;

pub use quantity::Quantity;

const TRUE_MINUS: &str = "\u{2212}";
const NO_VALUE: &str = "-";
const BASE_EXPONENT: i32 = 0;

/// A number ready to show: `value` stays in SI units, `text` is the number part, `unit` carries the prefix.
#[derive(Debug, Clone, PartialEq)]
pub struct Formatted {
    pub value: f64,
    pub text: String,
    pub unit: String,
}

pub fn format(value: f64, quantity: Quantity) -> Formatted {
    if !value.is_finite() {
        return Formatted {
            value,
            text: NO_VALUE.to_owned(),
            unit: scale::unit(quantity, BASE_EXPONENT),
        };
    }
    let (text, exponent) = scale::render(value, quantity);
    Formatted {
        value,
        text: text.replace('-', TRUE_MINUS),
        unit: scale::unit(quantity, exponent),
    }
}
