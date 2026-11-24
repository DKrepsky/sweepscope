use std::f64::consts::TAU;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Window {
    Rectangular,
    Hann,
    Hamming,
    Blackman,
}

impl Window {
    /// Periodic (DFT-even) form, so a bin-centred sine does not leak.
    pub(crate) fn weights(self, len: usize) -> Vec<f64> {
        (0..len)
            .map(|n| {
                let phase = TAU * n as f64 / len as f64;
                match self {
                    Self::Rectangular => 1.0,
                    Self::Hann => 0.5 - 0.5 * phase.cos(),
                    Self::Hamming => 0.54 - 0.46 * phase.cos(),
                    Self::Blackman => 0.42 - 0.5 * phase.cos() + 0.08 * (2.0 * phase).cos(),
                }
            })
            .collect()
    }

    /// Last bin of the DC and main-lobe region: 1, 2, 2, 3.
    pub(crate) fn last_skipped_bin(self) -> usize {
        match self {
            Self::Rectangular => 1,
            Self::Hann | Self::Hamming => 2,
            Self::Blackman => 3,
        }
    }
}
