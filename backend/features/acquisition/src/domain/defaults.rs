/// First-run values (spec decision D5); nothing persists yet.
#[derive(Debug, Clone, PartialEq)]
pub struct Defaults {
    pub range_volts_per_div: f64,
    pub time_base_seconds_per_div: f64,
    pub position_divs: f64,
    pub trigger_fraction: f64,
    /// Spec decision D4: a run in progress at unplug resumes after the restore.
    pub resume_after_reconnect: bool,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            range_volts_per_div: 1.0,
            time_base_seconds_per_div: 200e-6,
            position_divs: 0.0,
            trigger_fraction: 0.5,
            resume_after_reconnect: true,
        }
    }
}
