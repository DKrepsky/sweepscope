//! The decision state machine. Rules are in ADR 0020 item 2.

use crate::{FailReason, Observation, Plan, Step, Tables};

const MAX_PROBES: u8 = 14;
const PROBES_PER_RUNG: u8 = 3;
const MAX_VERTICAL_RETRIES: u8 = 2;
const NO_SIGNAL_COUNTS: f64 = 3.0;
const UNSETTLED_COUNTS: f64 = 6.0;
const MIN_SCREEN_PERIODS: f64 = 1.5;
const MAX_SCREEN_PERIODS: f64 = 7.0;
const FLAT_VOLTS_PER_DIV: f64 = 1.0;
const FLAT_SECONDS_PER_DIV: f64 = 1e-3;

#[derive(Debug, Clone, Copy)]
enum Stage {
    /// Looking for a period at one of the fixed time bases.
    Rung { rung: usize, probes: u8 },
    /// Probing at the chosen time base to settle the ranges and confirm the period.
    Final { vertical_retries: u8, corrected: bool },
}

pub struct Autoset {
    tables: Tables,
    enabled: [bool; 4],
    plan: Plan,
    probes: u8,
    probe_limit: u8,
    stage: Stage,
    reference: usize,
}

impl Autoset {
    pub fn new(tables: Tables, enabled: [bool; 4], start: Plan) -> Self {
        let reference = enabled.iter().position(|on| *on).unwrap_or(0);
        Self {
            tables,
            enabled,
            plan: start,
            probes: 0,
            probe_limit: MAX_PROBES,
            stage: Stage::Rung { rung: 0, probes: 1 },
            reference,
        }
    }

    pub fn first(&mut self) -> Step {
        let mut plan = self.plan.clone();
        plan.time_base_index = self.rung_index(0);
        self.account(Step::Probe(plan))
    }

    pub fn next(&mut self, observation: &Observation) -> Step {
        let step = match self.stage {
            Stage::Rung { rung, probes } => self.after_rung_probe(rung, probes, observation),
            Stage::Final {
                vertical_retries,
                corrected,
            } => self.after_final_probe(vertical_retries, corrected, observation),
        };
        self.account(step)
    }

    fn account(&mut self, step: Step) -> Step {
        let Step::Probe(plan) = step else { return step };
        if self.probes >= self.probe_limit {
            return Step::Failed(FailReason::ProbeBudget);
        }
        self.probes += 1;
        self.plan = plan.clone();
        Step::Probe(plan)
    }

    fn after_rung_probe(&mut self, rung: usize, probes: u8, observation: &Observation) -> Step {
        let adjusted = self.settled_ranges(observation);
        if let Some(plan) = adjusted.clone().filter(|_| probes < PROBES_PER_RUNG) {
            self.stage = Stage::Rung {
                rung,
                probes: probes + 1,
            };
            return Step::Probe(self.with_trigger(plan, observation));
        }
        let base = adjusted.unwrap_or_else(|| self.plan.clone());
        if let Some((channel, period)) = self.first_period(observation) {
            self.reference = channel;
            return self.enter_final(base, period, observation);
        }
        if rung + 1 < self.tables.rungs().len() {
            self.stage = Stage::Rung {
                rung: rung + 1,
                probes: 1,
            };
            let mut plan = self.with_trigger(base, observation);
            plan.time_base_index = self.rung_index(rung + 1);
            return Step::Probe(plan);
        }
        self.end_of_search(observation)
    }

    fn rung_index(&self, rung: usize) -> u8 {
        narrow(self.tables.rungs()[rung])
    }

    fn enter_final(&mut self, base: Plan, period_s: f64, observation: &Observation) -> Step {
        let mut plan = self.with_trigger(base, observation);
        plan.time_base_index = narrow(self.tables.time_base_for(period_s));
        self.stage = Stage::Final {
            vertical_retries: 0,
            corrected: false,
        };
        Step::Probe(plan)
    }

    fn first_period(&self, observation: &Observation) -> Option<(usize, f64)> {
        self.enabled_channels().find_map(|channel| {
            let period = observation.channels[channel]?.period_s?;
            Some((channel, period))
        })
    }

    /// A clipped channel goes up; a nearly flat one goes down towards five divisions. Positions follow the swing.
    fn settled_ranges(&self, observation: &Observation) -> Option<Plan> {
        let mut next = self.plan.clone();
        let mut changed = false;
        for channel in self.enabled_channels() {
            let Some(seen) = observation.channels[channel] else {
                continue;
            };
            let range = usize::from(self.plan.range_index[channel]);
            let wanted = if seen.saturated() {
                self.tables.raised(range)
            } else if seen.peak_to_peak_counts() < UNSETTLED_COUNTS && range > self.tables.floor_range() {
                self.tables.lowered_for(range, seen.peak_to_peak_counts())
            } else {
                range
            };
            if wanted == range {
                continue;
            }
            changed = true;
            next.range_index[channel] = narrow(wanted);
            next.position_divs[channel] = if seen.saturated() {
                0.0
            } else {
                let mid = self.tables.mid_volts(&seen, range, self.plan.position_divs[channel]);
                self.tables.centring_position(mid, wanted)
            };
        }
        changed.then_some(next)
    }

    fn end_of_search(&self, observation: &Observation) -> Step {
        let flat = self.enabled_channels().all(|channel| {
            observation.channels[channel]
                .is_none_or(|seen| !seen.saturated() && seen.peak_to_peak_counts() < NO_SIGNAL_COUNTS)
        });
        if flat {
            Step::NoSignal(self.flat_plan())
        } else {
            Step::Failed(FailReason::NoPeriod)
        }
    }

    fn flat_plan(&self) -> Plan {
        let mut plan = self.plan.clone();
        let one_volt = self
            .tables
            .range_volts_per_div
            .iter()
            .position(|volts| *volts >= FLAT_VOLTS_PER_DIV)
            .unwrap_or(0);
        for channel in self.enabled_channels() {
            plan.range_index[channel] = narrow(one_volt);
            plan.position_divs[channel] = 0.0;
        }
        plan.time_base_index = narrow(self.tables.nearest_time_base(FLAT_SECONDS_PER_DIV));
        plan.trigger_source = narrow(self.reference);
        plan.trigger_level_volts = 0.0;
        plan
    }

    fn after_final_probe(&mut self, vertical_retries: u8, corrected: bool, observation: &Observation) -> Step {
        let (plan, saturated) = self.final_vertical(observation);
        let ranges_changed = plan.range_index != self.plan.range_index;
        if ranges_changed && vertical_retries < MAX_VERTICAL_RETRIES {
            self.stage = Stage::Final {
                vertical_retries: vertical_retries + 1,
                corrected,
            };
            return Step::Probe(plan);
        }
        if saturated {
            return Step::Failed(FailReason::Clipped);
        }
        let period = observation.channels[self.reference].and_then(|seen| seen.period_s);
        let on_screen = period.map(|seconds| {
            self.tables
                .periods_on_screen(usize::from(plan.time_base_index), seconds)
        });
        match (period, on_screen) {
            (_, Some(periods)) if (MIN_SCREEN_PERIODS..=MAX_SCREEN_PERIODS).contains(&periods) => Step::Done(plan),
            (Some(seconds), _) if !corrected => {
                self.stage = Stage::Final {
                    vertical_retries,
                    corrected: true,
                };
                let mut again = plan;
                again.time_base_index = narrow(self.tables.time_base_for(seconds));
                Step::Probe(again)
            }
            _ => Step::Failed(FailReason::UnstablePeriod),
        }
    }

    /// The plan with the smallest range per channel that shows its swing within seven divisions, and whether any channel clipped.
    fn final_vertical(&self, observation: &Observation) -> (Plan, bool) {
        let mut plan = self.plan.clone();
        let mut saturated = false;
        for channel in self.enabled_channels() {
            let Some(seen) = observation.channels[channel] else {
                continue;
            };
            let range = usize::from(self.plan.range_index[channel]);
            if seen.saturated() {
                saturated = true;
                plan.range_index[channel] = narrow(self.tables.raised(range));
                plan.position_divs[channel] = 0.0;
            } else if seen.peak_to_peak_counts() >= NO_SIGNAL_COUNTS {
                let mid = self.tables.mid_volts(&seen, range, self.plan.position_divs[channel]);
                let wanted = self.tables.range_fitting(self.tables.swing_volts(&seen, range), mid);
                plan.range_index[channel] = narrow(wanted);
                plan.position_divs[channel] = self.tables.centring_position(mid, wanted);
            }
        }
        (self.with_trigger(plan, observation), saturated)
    }

    /// Edge trigger on the reference channel at the middle of its swing, in volts at the input.
    fn with_trigger(&self, mut plan: Plan, observation: &Observation) -> Plan {
        plan.trigger_source = narrow(self.reference);
        let seen = observation.channels[self.reference].filter(|seen| !seen.saturated());
        if let Some(seen) = seen {
            let range = usize::from(self.plan.range_index[self.reference]);
            plan.trigger_level_volts = self
                .tables
                .mid_volts(&seen, range, self.plan.position_divs[self.reference]);
        }
        plan
    }

    fn enabled_channels(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.enabled.len()).filter(|channel| self.enabled[*channel])
    }
}

fn narrow(index: usize) -> u8 {
    u8::try_from(index).unwrap_or(u8::MAX)
}

#[cfg(test)]
impl Autoset {
    fn with_probe_limit(mut self, limit: u8) -> Self {
        self.probe_limit = limit;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ChannelObservation;

    fn small_tables() -> Tables {
        Tables {
            range_volts_per_div: vec![0.01, 0.1, 1.0],
            range_verified: vec![true; 3],
            max_position_divs: vec![4.0; 3],
            time_base_s_per_div: vec![1e-6, 1e-4, 2e-2, 1.0],
            roll_from_index: 3,
            counts_per_div: 25.0,
            record_len: 4096,
            samples_per_div: 250.0,
        }
    }

    fn start() -> Plan {
        Plan {
            range_index: [2; 4],
            position_divs: [0.0; 4],
            time_base_index: 2,
            trigger_source: 0,
            trigger_level_volts: 0.0,
        }
    }

    #[test]
    fn given_a_probe_limit_of_one_when_a_second_probe_is_needed_then_the_run_fails_on_the_budget() {
        let clipped = ChannelObservation {
            min_code: 255,
            max_code: 255,
            clipped: true,
            period_s: None,
        };
        let observation = Observation {
            channels: [Some(clipped), None, None, None],
            sample_interval_s: 4e-9,
        };
        let mut autoset = Autoset::new(small_tables(), [true, false, false, false], start()).with_probe_limit(1);

        let first = autoset.first();
        let second = autoset.next(&observation);

        assert!(matches!(first, Step::Probe(_)));
        assert_eq!(second, Step::Failed(FailReason::ProbeBudget));
    }
}
