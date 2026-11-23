/// What a number measures; decides the unit and whether a prefix applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quantity {
    Volts,
    VoltsPerDiv,
    Seconds,
    SecondsPerDiv,
    Hertz,
    SamplesPerSecond,
    Percent,
    Divisions,
    Points,
}

/// Powers of ten, in steps of three, a quantity may use for its prefix.
pub(crate) struct PrefixRange {
    pub lowest: i32,
    pub highest: i32,
}

pub(crate) enum Digits {
    ThreeSignificant,
    /// Scale steps (1-2-5) are shown as they are, without trailing zeros.
    Exact,
}

impl Quantity {
    /// Unit symbol and the suffix that follows the prefixed symbol (`V` and `/div` give `mV/div`).
    pub(crate) fn symbol_and_suffix(self) -> (&'static str, &'static str) {
        match self {
            Self::Volts => ("V", ""),
            Self::VoltsPerDiv => ("V", "/div"),
            Self::Seconds => ("s", ""),
            Self::SecondsPerDiv => ("s", "/div"),
            Self::Hertz => ("Hz", ""),
            Self::SamplesPerSecond => ("S", "/s"),
            Self::Percent => ("%", ""),
            Self::Divisions => ("div", ""),
            Self::Points => ("pts", ""),
        }
    }

    pub(crate) fn prefix_range(self) -> PrefixRange {
        match self {
            Self::Percent | Self::Divisions => PrefixRange { lowest: 0, highest: 0 },
            Self::Points => PrefixRange { lowest: 0, highest: 9 },
            _ => PrefixRange {
                lowest: -12,
                highest: 9,
            },
        }
    }

    pub(crate) fn digits(self) -> Digits {
        match self {
            Self::VoltsPerDiv | Self::SecondsPerDiv => Digits::Exact,
            _ => Digits::ThreeSignificant,
        }
    }
}
