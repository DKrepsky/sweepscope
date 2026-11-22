use std::sync::Arc;
use std::time::{Duration, Instant};

use sweepscope_kernel::Clock;

use super::control::SimulatorControl;
use super::edge::{self, Direction};
use super::profile::{self, RECORD_LEN};
use super::signal;
use super::timing::{self, POLL_INTERVAL};
use crate::{
    AcquisitionSetup, Capabilities, ChannelData, DeviceError, DeviceIdentity, Frame, ScopeDevice, Slope, Sweep,
    TriggerOutcome,
};

/// What `configure` derives once, so capture and roll do not recompute it.
#[derive(Clone)]
struct Plan {
    setup: AcquisitionSetup,
    enabled: u8,
    interval_s: f64,
    counts_per_div: f64,
}

/// Simulated instrument. Simulated time is the injected clock's time since the device was opened.
pub struct SimulatedDevice {
    clock: Arc<dyn Clock>,
    control: SimulatorControl,
    epoch: Instant,
    generation: u64,
    identity: DeviceIdentity,
    capabilities: Capabilities,
    plan: Option<Plan>,
    roll_pointer: Option<Duration>,
}

impl SimulatedDevice {
    pub fn new(clock: Arc<dyn Clock>, control: SimulatorControl) -> Self {
        Self {
            epoch: clock.now(),
            generation: control.generation(),
            clock,
            control,
            identity: profile::identity(),
            capabilities: profile::capabilities(),
            plan: None,
            roll_pointer: None,
        }
    }

    fn ensure_connected(&self) -> Result<(), DeviceError> {
        if self.control.is_reachable(self.generation) {
            Ok(())
        } else {
            Err(DeviceError::Disconnected)
        }
    }

    fn plan(&self) -> Result<Plan, DeviceError> {
        self.plan
            .clone()
            .ok_or_else(|| DeviceError::InvalidSetting("the device is not configured".to_owned()))
    }

    fn sim_now(&self) -> Duration {
        self.clock.now().saturating_duration_since(self.epoch)
    }

    /// Sleeps in poll-sized slices and notices an unplug between them, like the driver's poll loop.
    fn wait(&self, total: Duration) -> Result<(), DeviceError> {
        let mut left = total;
        while !left.is_zero() {
            self.ensure_connected()?;
            let slice = left.min(POLL_INTERVAL);
            self.clock.sleep(slice);
            left -= slice;
        }
        self.ensure_connected()
    }

    fn validate(&self, setup: &AcquisitionSetup) -> Result<(), DeviceError> {
        let invalid = |reason: &str| Err(DeviceError::InvalidSetting(reason.to_owned()));
        if usize::from(setup.time_base_index) >= self.capabilities.time_bases.len() {
            return invalid("time base index is outside the table");
        }
        if !setup.channels.iter().any(|channel| channel.enabled) {
            return invalid("no channel is enabled");
        }
        let range_count = self.capabilities.vertical_ranges.len();
        if setup
            .channels
            .iter()
            .any(|channel| channel.enabled && usize::from(channel.range_index) >= range_count)
        {
            return invalid("range index is outside the table");
        }
        if !setup.trigger.level_volts.is_finite() || setup.channels.iter().any(|c| !c.position_divs.is_finite()) {
            return invalid("position and trigger level must be finite numbers");
        }
        let source = setup.trigger.source;
        if !setup
            .channels
            .get(usize::from(source))
            .is_some_and(|channel| channel.enabled)
        {
            return invalid("trigger source is not an enabled channel");
        }
        if !(0.0..=100.0).contains(&setup.trigger.horizontal_position_percent) {
            return invalid("horizontal position is outside 0 to 100 percent");
        }
        Ok(())
    }

    fn make_plan(&self, setup: &AcquisitionSetup) -> Plan {
        let enabled = setup.channels.iter().filter(|channel| channel.enabled).count() as u8;
        let counts_per_div = profile::counts_per_div(profile::adc_channels(enabled));
        Plan {
            setup: setup.clone(),
            enabled,
            interval_s: self.capabilities.sample_interval_s(enabled, setup.time_base_index),
            counts_per_div,
        }
    }

    /// Time of the first trigger edge in `[armed, armed + window]`, in simulated seconds.
    fn find_edge(&self, plan: &Plan, armed: f64, window: Duration) -> Option<f64> {
        let trigger = &plan.setup.trigger;
        let kind = self.control.signal(usize::from(trigger.source));
        let period = 1.0 / signal::FREQUENCY_HZ;
        // The waveform repeats, so an edge that exists shows up within two periods.
        let to = armed + window.as_secs_f64().min(2.0 * period);
        let step = plan.interval_s.min(period / 200.0).max(period / 100_000.0);
        let direction = match trigger.slope {
            Slope::Rising => Direction::Rising,
            Slope::Falling => Direction::Falling,
        };
        edge::find_crossing(
            |t| signal::clean(kind, t),
            trigger.level_volts,
            direction,
            armed,
            to,
            step,
        )
    }

    fn render(&self, plan: &Plan, start: f64, count: usize, trigger: TriggerOutcome) -> Frame {
        Frame {
            channels: std::array::from_fn(|channel| {
                plan.setup.channels[channel]
                    .enabled
                    .then(|| self.channel_data(plan, channel, start, count))
            }),
            counts_per_div: plan.counts_per_div,
            sample_interval_s: plan.interval_s,
            trigger,
        }
    }

    fn channel_data(&self, plan: &Plan, channel: usize, start: f64, count: usize) -> ChannelData {
        let setup = &plan.setup.channels[channel];
        let volts_per_div = self.capabilities.vertical_ranges[usize::from(setup.range_index)].volts_per_div;
        let zero_code = profile::zero_code(setup.position_divs, plan.counts_per_div);
        let kind = self.control.signal(channel);
        let codes = (0..count)
            .map(|i| {
                let volts = signal::sample(kind, start + i as f64 * plan.interval_s);
                to_code(volts, volts_per_div, zero_code, plan.counts_per_div)
            })
            .collect();
        ChannelData {
            codes,
            volts_per_div,
            zero_code,
        }
    }

    fn captured(&self, plan: &Plan, armed: f64, crossing: Option<f64>) -> Frame {
        let trigger = &plan.setup.trigger;
        match crossing {
            Some(at) => {
                let index = edge::trigger_index(trigger.horizontal_position_percent, RECORD_LEN);
                let start = edge::record_start(at, index, plan.interval_s);
                self.render(
                    plan,
                    start,
                    RECORD_LEN as usize,
                    TriggerOutcome::Triggered { sample_index: index },
                )
            }
            None => self.render(plan, armed, RECORD_LEN as usize, TriggerOutcome::Auto),
        }
    }
}

fn to_code(volts: f64, volts_per_div: f64, zero_code: f64, counts_per_div: f64) -> u8 {
    (zero_code + volts / volts_per_div * counts_per_div)
        .round()
        .clamp(0.0, 255.0) as u8
}

impl ScopeDevice for SimulatedDevice {
    fn identity(&self) -> &DeviceIdentity {
        &self.identity
    }

    fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    fn configure(&mut self, setup: &AcquisitionSetup) -> Result<(), DeviceError> {
        self.ensure_connected()?;
        self.validate(setup)?;
        self.plan = Some(self.make_plan(setup));
        self.roll_pointer = None;
        Ok(())
    }

    fn capture(&mut self, timeout: Duration) -> Result<Frame, DeviceError> {
        self.ensure_connected()?;
        let plan = self.plan()?;
        let armed = self.sim_now().as_secs_f64();
        let record = timing::record_duration(plan.interval_s, RECORD_LEN);
        let normal = plan.setup.trigger.sweep == Sweep::Normal;
        let crossing = self.find_edge(&plan, armed, if normal { timeout } else { record });
        let acquisition = crossing.map_or(record, |at| Duration::from_secs_f64((at - armed).max(0.0)) + record);
        let cycle = timing::frame_cycle(acquisition);
        if normal && (crossing.is_none() || cycle > timeout) {
            self.wait(timeout)?;
            return Err(DeviceError::Timeout);
        }
        self.wait(cycle)?;
        Ok(self.captured(&plan, armed, crossing))
    }

    fn start_roll(&mut self) -> Result<(), DeviceError> {
        self.ensure_connected()?;
        let plan = self.plan()?;
        if !self.capabilities.time_bases[usize::from(plan.setup.time_base_index)].streaming {
            return Err(DeviceError::InvalidSetting(
                "roll needs a time base of 500 ms per division or slower".to_owned(),
            ));
        }
        self.roll_pointer = Some(self.sim_now());
        Ok(())
    }

    fn poll_roll(&mut self) -> Result<Frame, DeviceError> {
        self.ensure_connected()?;
        let plan = self.plan()?;
        let pointer = self
            .roll_pointer
            .ok_or_else(|| DeviceError::InvalidSetting("roll was not started".to_owned()))?;
        let now = self.sim_now();
        let span = timing::roll_span(profile::adc_channels(plan.enabled), plan.interval_s);
        if self.control.take_overrun() || timing::roll_overruns(now - pointer, span) {
            self.roll_pointer = Some(now);
            return Err(DeviceError::Overrun);
        }
        let interval = Duration::from_nanos((plan.interval_s * 1e9).round() as u64);
        let count = (now - pointer).as_nanos() / interval.as_nanos().max(1);
        self.roll_pointer = Some(pointer + interval * u32::try_from(count).unwrap_or(u32::MAX));
        Ok(self.render(&plan, pointer.as_secs_f64(), count as usize, TriggerOutcome::None))
    }

    fn stop(&mut self) -> Result<(), DeviceError> {
        self.ensure_connected()?;
        self.roll_pointer = None;
        Ok(())
    }
}
