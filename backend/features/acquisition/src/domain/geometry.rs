//! Where the ten visible divisions sit inside the hardware record (implementation spec 3.2).

const SCREEN_DIVS: f64 = 10.0;
const MIN_WINDOW_LEN: f64 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geometry {
    pub samples_per_div: f64,
    pub window_len: usize,
    pub window_start: usize,
    pub record_divs: f64,
}

/// The window is centred in the record and shrinks below ten divisions when the record is shorter.
pub fn geometry(record_len: usize, sample_interval_s: f64, seconds_per_div: f64) -> Geometry {
    let samples_per_div = seconds_per_div / sample_interval_s;
    let wanted = (SCREEN_DIVS * samples_per_div).round() + 1.0;
    let window_len = wanted.max(MIN_WINDOW_LEN).min(record_len as f64) as usize;
    Geometry {
        samples_per_div,
        window_len,
        window_start: record_len.saturating_sub(window_len - 1) / 2,
        record_divs: record_len as f64 / samples_per_div,
    }
}

/// Maps a position on the screen (0 to 1) to the hardware trigger position in percent of the record.
pub fn record_percent(g: &Geometry, record_len: usize, screen_fraction: f64) -> f64 {
    let in_window = screen_fraction * (g.window_len - 1) as f64;
    100.0 * (g.window_start as f64 + in_window) / record_len as f64
}

pub fn trigger_index_in_window(g: &Geometry, sample_index: usize) -> Option<usize> {
    let index = sample_index.checked_sub(g.window_start)?;
    (index < g.window_len).then_some(index)
}
