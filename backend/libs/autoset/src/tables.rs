//! Range and time base arithmetic over the capability tables.

use crate::{ChannelObservation, Tables};

pub(crate) const SCREEN_CENTRE_CODE: f64 = 128.0;
const SATURATED_LOW: u8 = 0;
const SATURATED_HIGH: u8 = 255;
const SENSITIVE_FLOOR_VOLTS_PER_DIV: f64 = 0.01;
const TARGET_DIVS: f64 = 5.0;
const MAX_FIT_DIVS: f64 = 7.0;
const HALF_SCREEN_DIVS: f64 = 4.0;
const MIN_PREFERRED_DIVS: f64 = 3.0;
const CLIP_RAISE_STEPS: usize = 2;
const SCREEN_DIVS: f64 = 10.0;
const FIRST_PERIOD_FRACTION: f64 = 0.2;
const LAST_PERIOD_FRACTION: f64 = 0.6;
const PREFERRED_PERIOD_FRACTION: f64 = 0.4;
const RUNG_SECONDS_PER_DIV: [f64; 3] = [1e-6, 100e-6, 20e-3];
const TABLE_TOLERANCE: f64 = 1e-6;

impl ChannelObservation {
    pub(crate) fn saturated(&self) -> bool {
        self.clipped || self.min_code == SATURATED_LOW || self.max_code == SATURATED_HIGH
    }

    pub(crate) fn peak_to_peak_counts(&self) -> f64 {
        f64::from(self.max_code) - f64::from(self.min_code)
    }

    fn mid_code(&self) -> f64 {
        (f64::from(self.max_code) + f64::from(self.min_code)) / 2.0
    }
}

impl Tables {
    pub(crate) fn last_range(&self) -> usize {
        self.range_volts_per_div.len().saturating_sub(1)
    }

    pub(crate) fn volts_per_div(&self, range: usize) -> f64 {
        self.range_volts_per_div.get(range).copied().unwrap_or(1.0)
    }

    /// The most sensitive range the search uses: 10 mV/div or the nearest table value below.
    pub(crate) fn floor_range(&self) -> usize {
        let limit = SENSITIVE_FLOOR_VOLTS_PER_DIV * (1.0 + TABLE_TOLERANCE);
        self.range_volts_per_div
            .iter()
            .rposition(|volts| *volts <= limit)
            .unwrap_or(0)
    }

    pub(crate) fn raised(&self, range: usize) -> usize {
        (range + CLIP_RAISE_STEPS).min(self.last_range())
    }

    /// The range that would show the observed swing at about five divisions, never below the floor.
    pub(crate) fn lowered_for(&self, range: usize, peak_to_peak_counts: f64) -> usize {
        let swing_volts = peak_to_peak_counts / self.counts_per_div * self.volts_per_div(range);
        let target = swing_volts / TARGET_DIVS;
        let wanted = (self.floor_range()..=range).find(|index| self.volts_per_div(*index) >= target);
        wanted.unwrap_or(range).min(range)
    }

    pub(crate) fn swing_volts(&self, observation: &ChannelObservation, range: usize) -> f64 {
        observation.peak_to_peak_counts() / self.counts_per_div * self.volts_per_div(range)
    }

    /// Level of the middle of the swing in volts at the input.
    pub(crate) fn mid_volts(&self, observation: &ChannelObservation, range: usize, position_divs: f64) -> f64 {
        let divs = (observation.mid_code() - SCREEN_CENTRE_CODE) / self.counts_per_div - position_divs;
        divs * self.volts_per_div(range)
    }

    /// Smallest range showing the swing within seven divisions with its centre reachable by the position; a verified
    /// neighbour wins while it keeps three divisions.
    pub(crate) fn range_fitting(&self, swing_volts: f64, mid_volts: f64) -> usize {
        let divs_at = |index: usize| swing_volts / self.volts_per_div(index);
        let fits = |index: usize| {
            divs_at(index) <= MAX_FIT_DIVS && self.visible_half_swing(index, swing_volts, mid_volts) <= HALF_SCREEN_DIVS
        };
        let smallest = (0..=self.last_range())
            .find(|index| fits(*index))
            .unwrap_or(self.last_range());
        let next = smallest + 1;
        let verified = |index: usize| self.range_verified.get(index).copied().unwrap_or(false);
        let neighbour_better = next <= self.last_range()
            && !verified(smallest)
            && verified(next)
            && fits(next)
            && divs_at(next) >= MIN_PREFERRED_DIVS;
        if neighbour_better {
            next
        } else {
            smallest
        }
    }

    /// Divisions from the screen centre to the far edge of the swing once the position has done what it can.
    fn visible_half_swing(&self, range: usize, swing_volts: f64, mid_volts: f64) -> f64 {
        let reach = self.max_position_divs.get(range).copied().unwrap_or(0.0);
        let volts_per_div = self.volts_per_div(range);
        let uncentred = (mid_volts.abs() / volts_per_div - reach).max(0.0);
        uncentred + swing_volts / volts_per_div / 2.0
    }

    pub(crate) fn centring_position(&self, mid_volts: f64, range: usize) -> f64 {
        let reach = self.max_position_divs.get(range).copied().unwrap_or(0.0);
        (-mid_volts / self.volts_per_div(range)).clamp(-reach, reach)
    }

    /// Time base indexes of the three search rungs: 1 us, 100 us and 20 ms per division.
    pub(crate) fn rungs(&self) -> [usize; 3] {
        RUNG_SECONDS_PER_DIV.map(|seconds| self.nearest_time_base(seconds))
    }

    pub(crate) fn nearest_time_base(&self, seconds: f64) -> usize {
        self.usable_time_bases()
            .min_by(|a, b| {
                self.log_distance(*a, seconds)
                    .total_cmp(&self.log_distance(*b, seconds))
            })
            .unwrap_or(0)
    }

    /// The step showing two to six periods in ten divisions, closest to four.
    pub(crate) fn time_base_for(&self, period_s: f64) -> usize {
        let within = |index: &usize| {
            let step = self.time_base_s_per_div[*index];
            step >= FIRST_PERIOD_FRACTION * period_s * (1.0 - TABLE_TOLERANCE)
                && step <= LAST_PERIOD_FRACTION * period_s * (1.0 + TABLE_TOLERANCE)
        };
        let preferred = PREFERRED_PERIOD_FRACTION * period_s;
        self.usable_time_bases()
            .filter(within)
            .min_by(|a, b| {
                self.log_distance(*a, preferred)
                    .total_cmp(&self.log_distance(*b, preferred))
            })
            .unwrap_or_else(|| self.nearest_time_base(preferred))
    }

    pub(crate) fn periods_on_screen(&self, time_base: usize, period_s: f64) -> f64 {
        SCREEN_DIVS * self.time_base_s_per_div[time_base] / period_s
    }

    fn usable_time_bases(&self) -> std::ops::Range<usize> {
        0..self.roll_from_index.min(self.time_base_s_per_div.len())
    }

    fn log_distance(&self, index: usize, seconds: f64) -> f64 {
        (self.time_base_s_per_div[index] / seconds).ln().abs()
    }
}
