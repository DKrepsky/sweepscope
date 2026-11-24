//! Level axis (8 divisions) and the tick texts; the frontend only looks them up (ADR 0018 item 3).
use engineering_format::{format, Quantity};

use crate::messages::{SpectrumScale, Tick};

const DIVISIONS: usize = 8;
const FREQUENCY_STEPS: usize = 5;
const DBV_TOP: f64 = 10.0;
const DBV_PER_DIV: f64 = 10.0;
const DBV_FLOOR: f64 = -200.0;
const SHRINK_BELOW: f64 = 0.25;
const SMALLEST_LINEAR_TOP: f64 = 1e-4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Axis {
    pub scale: SpectrumScale,
    pub top: f64,
}

impl Axis {
    pub fn dbv() -> Self {
        Self {
            scale: SpectrumScale::Dbv,
            top: DBV_TOP,
        }
    }

    pub fn linear(top: f64) -> Self {
        Self {
            scale: SpectrumScale::Linear,
            top,
        }
    }

    pub fn units_per_div(&self) -> f64 {
        match self.scale {
            SpectrumScale::Dbv => DBV_PER_DIV,
            SpectrumScale::Linear => self.top / DIVISIONS as f64,
        }
    }

    /// The level to show for an rms value, and its distance from the top edge in divisions.
    pub fn place(&self, rms: f64) -> (f32, f32) {
        let level = match self.scale {
            SpectrumScale::Dbv => (20.0 * rms.log10()).max(DBV_FLOOR),
            SpectrumScale::Linear => rms,
        };
        let divisions = (self.top - level) / self.units_per_div();
        (level as f32, divisions as f32)
    }

    pub fn top_label(&self) -> String {
        self.level_text(self.top, true)
    }

    pub fn ticks_y(&self) -> Vec<Tick> {
        (0..=DIVISIONS)
            .map(|division| Tick {
                at: division as f64 / DIVISIONS as f64,
                text: self.level_text(self.top - division as f64 * self.units_per_div(), division == 0),
            })
            .collect()
    }

    fn level_text(&self, level: f64, with_unit: bool) -> String {
        match self.scale {
            SpectrumScale::Dbv => dbv_text(level, with_unit),
            SpectrumScale::Linear => value_text(level, Quantity::Volts),
        }
    }
}

pub fn ticks_x(span_hz: f64) -> Vec<Tick> {
    (0..=FREQUENCY_STEPS)
        .map(|step| {
            let at = step as f64 / FREQUENCY_STEPS as f64;
            Tick {
                at,
                text: value_text(span_hz * at, Quantity::Hertz),
            }
        })
        .collect()
}

pub fn value_text(value: f64, quantity: Quantity) -> String {
    let formatted = format(value, quantity);
    format!("{} {}", formatted.text, formatted.unit)
}

fn dbv_text(level: f64, with_unit: bool) -> String {
    let rounded = level.round();
    let number = if rounded > 0.0 {
        format!("+{rounded}")
    } else {
        format!("{rounded}").replace('-', "\u{2212}")
    };
    if with_unit {
        format!("{number} dBV")
    } else {
        number
    }
}

/// The top of the linear axis: a 1-2-5 value that moves only when the largest level leaves the picture or shrinks to a quarter of it.
#[derive(Debug, Clone, Copy, Default)]
pub struct LinearTop {
    current: Option<f64>,
}

impl LinearTop {
    /// Before any spectrum the axis shows 1 V.
    pub fn current(&self) -> f64 {
        self.current.unwrap_or(1.0)
    }

    pub fn follow(&mut self, largest: f64) -> f64 {
        let top = match self.current {
            Some(top) if largest <= top && largest >= top * SHRINK_BELOW => top,
            Some(top) if largest < top * SHRINK_BELOW => step_at_least(largest * 2.0),
            _ => step_at_least(largest),
        };
        self.current = Some(top);
        top
    }
}

/// Smallest value of the 1-2-5 series that is at least `wanted`.
fn step_at_least(wanted: f64) -> f64 {
    let wanted = wanted.max(SMALLEST_LINEAR_TOP);
    let decade = 10f64.powi(wanted.log10().floor() as i32);
    [1.0, 2.0, 5.0, 10.0]
        .iter()
        .map(|mantissa| mantissa * decade)
        .find(|step| *step >= wanted * (1.0 - 1e-9))
        .unwrap_or(10.0 * decade)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_dbv_when_ticks_are_made_then_every_division_is_labelled_and_only_the_top_has_the_unit() {
        let texts: Vec<String> = Axis::dbv().ticks_y().into_iter().map(|tick| tick.text).collect();

        assert_eq!(texts.len(), 9);
        assert_eq!(texts[0], "+10 dBV");
        assert_eq!(texts[1], "0");
        assert_eq!(texts[2], "\u{2212}10");
        assert_eq!(texts[8], "\u{2212}70");
    }

    #[test]
    fn given_a_level_below_the_floor_when_placed_in_dbv_then_it_is_clamped_to_minus_200() {
        let (level, _) = Axis::dbv().place(0.0);

        assert_eq!(level, -200.0);
    }

    #[test]
    fn given_half_a_volt_in_dbv_when_placed_then_it_is_minus_6_dbv_and_1_6_divisions_below_the_top() {
        let (level, divisions) = Axis::dbv().place(0.5);

        assert!((f64::from(level) + 6.0206).abs() < 1e-3);
        assert!((f64::from(divisions) - 1.602).abs() < 1e-3);
    }

    #[test]
    fn given_a_linear_top_of_two_volts_when_ticks_are_made_then_they_are_volts_with_prefixes() {
        let texts: Vec<String> = Axis::linear(2.0).ticks_y().into_iter().map(|tick| tick.text).collect();

        assert_eq!(texts[0], "2.00 V");
        assert_eq!(texts[4], "1.00 V");
    }

    #[test]
    fn given_a_first_peak_when_following_then_the_top_is_the_next_1_2_5_value() {
        let mut top = LinearTop::default();

        assert_eq!(top.follow(0.7), 1.0);
    }

    #[test]
    fn given_a_peak_above_the_top_when_following_then_the_top_steps_up_once() {
        let mut top = LinearTop::default();
        top.follow(0.7);

        assert_eq!(top.follow(1.2), 2.0);
        assert_eq!(top.follow(1.2), 2.0);
    }

    #[test]
    fn given_a_peak_within_a_quarter_and_the_top_when_following_then_the_top_stays() {
        let mut top = LinearTop::default();
        top.follow(0.7);

        assert_eq!(top.follow(0.26), 1.0);
        assert_eq!(top.follow(0.99), 1.0);
    }

    #[test]
    fn given_a_peak_below_a_quarter_of_the_top_when_following_then_the_top_shrinks() {
        let mut top = LinearTop::default();
        top.follow(0.7);

        assert_eq!(top.follow(0.1), 0.2);
    }

    #[test]
    fn given_a_span_when_x_ticks_are_made_then_they_run_from_zero_to_the_span_every_two_divisions() {
        let ticks = ticks_x(25_000.0);

        assert_eq!(ticks.len(), 6);
        assert_eq!(ticks[0].at, 0.0);
        assert_eq!(ticks[5].at, 1.0);
        assert_eq!(ticks[5].text, "25.0 kHz");
    }
}
