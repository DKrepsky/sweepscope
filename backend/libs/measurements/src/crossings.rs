//! Mid-level crossings with hysteresis, positioned in fractional sample indexes.

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Crossing {
    /// Index of the sample before the crossing.
    pub index: usize,
    /// Fractional sample index of the mid-level crossing.
    pub position: f64,
    pub rising: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Side {
    Unknown,
    Low,
    High,
}

pub(crate) fn find(volts: &[f64], mid: f64, hysteresis: f64) -> Vec<Crossing> {
    let mut side = Side::Unknown;
    let mut crossings = Vec::new();
    for (at, &v) in volts.iter().enumerate() {
        let next = if v >= mid + hysteresis {
            Side::High
        } else if v <= mid - hysteresis {
            Side::Low
        } else {
            continue;
        };
        if side != Side::Unknown && side != next {
            let rising = next == Side::High;
            crossings.push(locate(volts, at, mid, rising));
        }
        side = next;
    }
    crossings
}

/// Walks back from the confirming sample to the sample pair that straddles `mid`.
fn locate(volts: &[f64], confirmed_at: usize, mid: f64, rising: bool) -> Crossing {
    let before_mid = |v: f64| if rising { v < mid } else { v > mid };
    let mut index = confirmed_at.saturating_sub(1);
    while index > 0 && !before_mid(volts[index]) {
        index -= 1;
    }
    let (a, b) = (volts[index], volts[index + 1]);
    let fraction = if (b - a).abs() > f64::EPSILON {
        ((mid - a) / (b - a)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    Crossing {
        index,
        position: index as f64 + fraction,
        rising,
    }
}

pub(crate) fn rising_positions(crossings: &[Crossing]) -> Vec<f64> {
    crossings.iter().filter(|c| c.rising).map(|c| c.position).collect()
}

/// Mean width of the complete pulses that start with a crossing of the given direction.
pub(crate) fn mean_pulse_width(crossings: &[Crossing], starts_rising: bool) -> Option<f64> {
    let widths: Vec<f64> = crossings
        .windows(2)
        .filter(|pair| pair[0].rising == starts_rising)
        .map(|pair| pair[1].position - pair[0].position)
        .collect();
    (!widths.is_empty()).then(|| widths.iter().sum::<f64>() / widths.len() as f64)
}

/// Period in samples: one slope shared by both edge types, each with its own intercept, fitted by least squares.
/// Needs two rising crossings; crossings alternate, so the cycle number counts the rising ones seen so far.
pub(crate) fn fitted_period(crossings: &[Crossing]) -> Option<f64> {
    if crossings.iter().filter(|c| c.rising).count() < 2 {
        return None;
    }
    let mut cycle = -1.0;
    let mut points = [Vec::new(), Vec::new()];
    for crossing in crossings {
        if crossing.rising {
            cycle += 1.0;
        }
        points[usize::from(crossing.rising)].push((cycle, crossing.position));
    }
    let (mut covariance, mut variance) = (0.0, 0.0);
    for group in &points {
        let n = group.len() as f64;
        let (mean_x, mean_y) = group.iter().fold((0.0, 0.0), |(x, y), p| (x + p.0 / n, y + p.1 / n));
        for &(x, y) in group {
            covariance += (x - mean_x) * (y - mean_y);
            variance += (x - mean_x) * (x - mean_x);
        }
    }
    (variance > 0.0).then(|| covariance / variance)
}
