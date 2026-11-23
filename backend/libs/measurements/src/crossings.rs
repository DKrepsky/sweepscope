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
