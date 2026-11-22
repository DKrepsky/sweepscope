use engineering_format::{format, Formatted, Quantity};

const MINUS: &str = "\u{2212}";

fn assert_shows(value: f64, quantity: Quantity, text: &str, unit: &str) {
    let Formatted {
        text: got_text,
        unit: got_unit,
        ..
    } = format(value, quantity);
    assert_eq!(
        (got_text.as_str(), got_unit.as_str()),
        (text, unit),
        "{value} {quantity:?}"
    );
}

#[test]
fn given_half_a_volt_per_div_when_formatted_then_it_reads_500_mv_per_div() {
    assert_shows(0.5, Quantity::VoltsPerDiv, "500", "mV/div");
}

#[test]
fn given_one_volt_per_div_when_formatted_then_trailing_zeros_are_not_kept() {
    assert_shows(1.0, Quantity::VoltsPerDiv, "1", "V/div");
}

#[test]
fn given_every_vertical_scale_step_when_formatted_then_the_step_value_is_exact() {
    let steps = [
        (0.002, "2", "mV/div"),
        (0.005, "5", "mV/div"),
        (0.01, "10", "mV/div"),
        (0.02, "20", "mV/div"),
        (0.05, "50", "mV/div"),
        (0.1, "100", "mV/div"),
        (0.2, "200", "mV/div"),
        (0.5, "500", "mV/div"),
        (1.0, "1", "V/div"),
        (2.0, "2", "V/div"),
        (5.0, "5", "V/div"),
        (10.0, "10", "V/div"),
    ];
    for (value, text, unit) in steps {
        assert_shows(value, Quantity::VoltsPerDiv, text, unit);
    }
}

#[test]
fn given_time_base_steps_when_formatted_then_the_step_value_is_exact() {
    let steps = [
        (2e-9, "2", "ns/div"),
        (200e-6, "200", "us/div"),
        (0.2, "200", "ms/div"),
        (0.5, "500", "ms/div"),
        (1.0, "1", "s/div"),
        (10.0, "10", "s/div"),
    ];
    for (value, text, unit) in steps {
        assert_shows(value, Quantity::SecondsPerDiv, text, unit);
    }
}

#[test]
fn given_a_fractional_scale_step_when_formatted_then_its_decimals_are_kept() {
    assert_shows(2.5e-6, Quantity::SecondsPerDiv, "2.5", "us/div");
}

#[test]
fn given_1_9_volts_when_formatted_then_it_reads_1_90_v() {
    assert_shows(1.9, Quantity::Volts, "1.90", "V");
}

#[test]
fn given_volts_across_magnitudes_when_formatted_then_three_significant_digits_are_shown() {
    assert_shows(0.0123, Quantity::Volts, "12.3", "mV");
    assert_shows(0.5, Quantity::Volts, "500", "mV");
    assert_shows(12.345, Quantity::Volts, "12.3", "V");
    assert_shows(123.456, Quantity::Volts, "123", "V");
    assert_shows(1234.0, Quantity::Volts, "1.23", "kV");
}

#[test]
fn given_a_sample_rate_of_250_mega_when_formatted_then_it_reads_250_ms_per_s() {
    assert_shows(250e6, Quantity::SamplesPerSecond, "250", "MS/s");
}

#[test]
fn given_sample_rates_when_formatted_then_the_prefix_follows_the_magnitude() {
    assert_shows(1e9, Quantity::SamplesPerSecond, "1.00", "GS/s");
    assert_shows(1.25e6, Quantity::SamplesPerSecond, "1.25", "MS/s");
    assert_shows(500.0, Quantity::SamplesPerSecond, "500", "S/s");
}

#[test]
fn given_two_nanoseconds_when_formatted_then_trailing_zeros_are_kept() {
    assert_shows(2e-9, Quantity::Seconds, "2.00", "ns");
}

#[test]
fn given_durations_when_formatted_then_the_unit_prefix_is_chosen_per_magnitude() {
    assert_shows(800e-9, Quantity::Seconds, "800", "ns");
    assert_shows(2.5e-3, Quantity::Seconds, "2.50", "ms");
    assert_shows(4.0e-6, Quantity::Seconds, "4.00", "us");
    assert_shows(1.5, Quantity::Seconds, "1.50", "s");
    assert_shows(3e-12, Quantity::Seconds, "3.00", "ps");
}

#[test]
fn given_a_frequency_when_formatted_then_it_reads_in_hertz_with_a_prefix() {
    assert_shows(1000.0, Quantity::Hertz, "1.00", "kHz");
    assert_shows(49.97, Quantity::Hertz, "50.0", "Hz");
}

#[test]
fn given_a_value_that_rounds_up_to_the_next_prefix_when_formatted_then_the_prefix_moves_up() {
    assert_shows(0.9996, Quantity::Volts, "1.00", "V");
    assert_shows(999.6e-6, Quantity::Seconds, "1.00", "ms");
}

#[test]
fn given_a_value_that_rounds_up_to_a_new_decade_when_formatted_then_three_digits_remain() {
    assert_shows(9.996, Quantity::Volts, "10.0", "V");
}

#[test]
fn given_a_value_below_one_in_a_prefixless_quantity_when_formatted_then_it_has_three_significant_digits() {
    assert_shows(0.5, Quantity::Divisions, "0.500", "div");
}

#[test]
fn given_zero_when_formatted_then_it_reads_0_00_with_the_base_unit() {
    assert_shows(0.0, Quantity::Volts, "0.00", "V");
    assert_shows(-0.0, Quantity::Seconds, "0.00", "s");
}

#[test]
fn given_zero_scale_when_formatted_then_it_reads_0_with_the_base_unit() {
    assert_shows(0.0, Quantity::VoltsPerDiv, "0", "V/div");
}

#[test]
fn given_a_negative_value_when_formatted_then_the_sign_is_the_true_minus() {
    assert_shows(-0.25, Quantity::Volts, &format!("{MINUS}250"), "mV");
}

#[test]
fn given_a_negative_division_count_when_formatted_then_the_sign_is_the_true_minus() {
    assert_shows(-1.5, Quantity::Divisions, &format!("{MINUS}1.50"), "div");
}

#[test]
fn given_not_a_number_when_formatted_then_the_text_is_a_dash() {
    assert_shows(f64::NAN, Quantity::Volts, "-", "V");
}

#[test]
fn given_infinity_when_formatted_then_the_text_is_a_dash_with_the_base_unit() {
    assert_shows(f64::INFINITY, Quantity::SecondsPerDiv, "-", "s/div");
    assert_shows(f64::NEG_INFINITY, Quantity::Hertz, "-", "Hz");
}

#[test]
fn given_a_percentage_when_formatted_then_it_has_no_prefix() {
    assert_shows(50.0, Quantity::Percent, "50.0", "%");
    assert_shows(100.0, Quantity::Percent, "100", "%");
    assert_shows(12.5, Quantity::Percent, "12.5", "%");
}

#[test]
fn given_a_division_position_when_formatted_then_it_reads_in_div() {
    assert_shows(-12.0, Quantity::Divisions, &format!("{MINUS}12.0"), "div");
}

#[test]
fn given_record_lengths_when_formatted_then_large_counts_take_a_prefix() {
    assert_shows(512.0, Quantity::Points, "512", "pts");
    assert_shows(4000.0, Quantity::Points, "4.00", "kpts");
    assert_shows(2_000_000.0, Quantity::Points, "2.00", "Mpts");
}

#[test]
fn given_a_value_when_formatted_then_the_raw_value_is_returned_unchanged() {
    assert_eq!(format(0.5, Quantity::VoltsPerDiv).value, 0.5);
}
