use crate::quantity::{Digits, PrefixRange, Quantity};

const GROUP: i32 = 3;
const SIGNIFICANT_DIGITS: i32 = 3;
const EXACT_DECIMALS: usize = 6;
const GROUP_TOLERANCE: f64 = 1e-12;
const NEXT_GROUP_AT: f64 = 1000.0;
const MAX_DECIMALS: i32 = 15;

pub(crate) fn unit(quantity: Quantity, exponent: i32) -> String {
    let (symbol, suffix) = quantity.symbol_and_suffix();
    format!("{}{symbol}{suffix}", prefix(exponent))
}

/// Number text (ASCII minus) and the exponent of the prefix that was used.
pub(crate) fn render(value: f64, quantity: Quantity) -> (String, i32) {
    let magnitude = value.abs();
    let range = quantity.prefix_range();
    let mut exponent = group_for(magnitude, &range);
    let mut text = number(magnitude, exponent, quantity);
    if overflows_group(&text, exponent, &range) {
        exponent += GROUP;
        text = number(magnitude, exponent, quantity);
    }
    let sign = if value < 0.0 && magnitude != 0.0 { "-" } else { "" };
    (format!("{sign}{text}"), exponent)
}

fn group_for(magnitude: f64, range: &PrefixRange) -> i32 {
    if magnitude == 0.0 {
        return 0;
    }
    (range.lowest / GROUP..=range.highest / GROUP)
        .rev()
        .map(|step| step * GROUP)
        .find(|&g| magnitude >= 10f64.powi(g) * (1.0 - GROUP_TOLERANCE))
        .unwrap_or(range.lowest)
}

fn overflows_group(text: &str, exponent: i32, range: &PrefixRange) -> bool {
    exponent + GROUP <= range.highest && text.parse::<f64>().is_ok_and(|n| n >= NEXT_GROUP_AT)
}

fn number(magnitude: f64, exponent: i32, quantity: Quantity) -> String {
    let scaled = magnitude / 10f64.powi(exponent);
    match quantity.digits() {
        Digits::Exact => exact(scaled),
        Digits::ThreeSignificant => three_significant(scaled),
    }
}

fn exact(scaled: f64) -> String {
    let text = format!("{scaled:.EXACT_DECIMALS$}");
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

fn three_significant(scaled: f64) -> String {
    let first = format!("{scaled:.*}", decimals_for(scaled));
    let rounded: f64 = first.parse().unwrap_or(scaled);
    let decimals = decimals_for(rounded);
    if decimals_for(scaled) == decimals {
        return first;
    }
    format!("{rounded:.decimals$}")
}

fn decimals_for(magnitude: f64) -> usize {
    if magnitude == 0.0 {
        return (SIGNIFICANT_DIGITS - 1) as usize;
    }
    let mut decimals = SIGNIFICANT_DIGITS - 1;
    let mut probe = magnitude;
    while probe >= 10.0 && decimals > 0 {
        probe /= 10.0;
        decimals -= 1;
    }
    while probe < 1.0 && decimals < MAX_DECIMALS {
        probe *= 10.0;
        decimals += 1;
    }
    decimals as usize
}

fn prefix(exponent: i32) -> &'static str {
    match exponent {
        -12 => "p",
        -9 => "n",
        -6 => "u",
        -3 => "m",
        3 => "k",
        6 => "M",
        9 => "G",
        _ => "",
    }
}
