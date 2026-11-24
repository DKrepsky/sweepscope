use std::cell::RefCell;
use std::time::Instant;

use sweepscope_kernel::contracts::{AcquisitionSetup, Capabilities, FrameContext, RollContext};
use sweepscope_kernel::KernelError;

use super::defaults::Defaults;
use super::math;
use super::meter::CaptureMeter;
use super::model::{nearest_index, Model, CHANNELS};
use super::pulse;
use super::setup_file::{self, SetupFile};
use super::view::{self, LAST_CHANNEL, NO_SCOPE};
use crate::{ChannelPatch, InstrumentFacts, InstrumentState, SettingTarget, SetupNotice, TriggerPatch};

const POSITION_STEP: f64 = 0.05;
const LEVEL_STEP_DIVS: f64 = 0.2;
pub(super) const SCREEN_HALF_HEIGHT_DIVS: f64 = 4.0;
pub(super) const PROBES: [f64; 3] = [1.0, 10.0, 100.0];

/// What the module must send to the device after a connection became ready.
#[derive(Debug, Clone, PartialEq)]
pub struct Restore {
    pub setup: AcquisitionSetup,
    pub resume: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Change {
    pub setup_changed: bool,
    pub state_changed: bool,
}

/// Owns what the user chose; every edit is validated first and applied whole or not at all.
pub struct Instrument {
    defaults: Defaults,
    caps: Option<Capabilities>,
    model: Option<Model>,
    unavailable: Option<String>,
    was_running: bool,
    /// The last setup, held until the first device is ready because the tables it must match are unknown before.
    pending_setup: Option<SetupFile>,
    notice: Option<SetupNotice>,
    autoset_running: bool,
}

/// Everything that makes a roll session: when any part changes the screen history is void and a new session begins.
#[derive(Debug, Clone, PartialEq)]
pub struct RollKey {
    setup: AcquisitionSetup,
    invert: [bool; CHANNELS],
    probe: [f64; CHANNELS],
}

/// What a loaded setup could not do; each line names a value that kept its current setting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupReport {
    pub warnings: Vec<String>,
}

type Edit<'a> = &'a dyn Fn(&Capabilities, &mut Model) -> Result<(), KernelError>;

impl Instrument {
    pub fn new(defaults: Defaults) -> Self {
        Self {
            defaults,
            caps: None,
            model: None,
            unavailable: Some(NO_SCOPE.to_owned()),
            was_running: false,
            pending_setup: None,
            notice: None,
            autoset_running: false,
        }
    }

    /// The setup to apply, like a load, when the first device becomes ready.
    pub fn hold_setup(&mut self, file: SetupFile) {
        self.pending_setup = Some(file);
    }

    /// Raises the one-time message of the instrument slice (ADR 0016 item 6).
    pub fn set_notice(&mut self, text: String) {
        let id = self.notice.as_ref().map_or(1, |n| n.id + 1);
        self.notice = Some(SetupNotice { id, text });
    }

    pub fn to_setup_file(&self) -> Option<SetupFile> {
        let (caps, model) = self.caps.as_ref().zip(self.model.as_ref())?;
        SetupFile::from_model(caps, model)
    }

    /// Needs a ready device. Values the device cannot do keep their setting and are named in the report.
    pub fn apply_setup_file(&mut self, file: &SetupFile) -> Result<SetupReport, KernelError> {
        let warnings = RefCell::new(Vec::new());
        self.edit(&|caps, model| {
            *warnings.borrow_mut() = setup_file::apply(file, caps, model);
            Ok(())
        })?;
        Ok(SetupReport {
            warnings: warnings.into_inner(),
        })
    }

    pub fn on_ready(&mut self, caps: Capabilities) -> Restore {
        let mut model = match self.model.take() {
            Some(mut kept) => {
                kept.revalidate(&caps);
                kept
            }
            None => Model::initial(&self.defaults, &caps),
        };
        if let Some(file) = self.pending_setup.take() {
            self.restore_last_setup(&file, &caps, &mut model);
        }
        let resume = (self.was_running || model.running) && self.defaults.resume_after_reconnect;
        model.running = resume;
        self.was_running = false;
        self.unavailable = None;
        let setup = model.setup.clone();
        self.model = Some(model);
        self.caps = Some(caps);
        Restore { setup, resume }
    }

    fn restore_last_setup(&mut self, file: &SetupFile, caps: &Capabilities, model: &mut Model) {
        let warnings = setup_file::apply(file, caps, model);
        if !warnings.is_empty() {
            let text = format!("The last setup was restored with changes: {}", warnings.join("; "));
            self.set_notice(text);
        }
    }

    pub fn on_lost(&mut self) {
        self.set_unavailable(NO_SCOPE);
    }

    /// The text every control shows while no device is ready: "Connecting", an error text, and so on.
    pub fn set_unavailable(&mut self, reason: &str) {
        if let Some(model) = &mut self.model {
            self.was_running |= model.running;
            model.running = false;
        }
        self.unavailable = Some(reason.to_owned());
    }

    /// Run or Stop; either one ends an armed Single.
    pub fn set_running(&mut self, on: bool) -> Result<Change, KernelError> {
        self.edit(&move |_, model| {
            model.running = on;
            model.single = false;
            Ok(())
        })
    }

    pub fn arm_single(&mut self) -> Result<Change, KernelError> {
        self.edit(&|caps, model| {
            if model.in_roll(caps) {
                return Err(KernelError::recoverable(view::SINGLE_IN_ROLL));
            }
            model.running = true;
            model.single = true;
            Ok(())
        })
    }

    pub fn single_armed(&self) -> bool {
        self.model.as_ref().is_some_and(|m| m.single)
    }

    /// The first accepted frame of a Single ends the run.
    pub fn finish_single(&mut self) -> Result<Change, KernelError> {
        self.set_running(false)
    }

    /// Set by the auto-setup run; locks Run, Single, Force and Auto-setup with the reason.
    pub fn set_autoset_running(&mut self, running: bool) {
        self.autoset_running = running;
    }

    pub fn ready(&self) -> bool {
        self.unavailable.is_none() && self.model.is_some()
    }

    pub fn in_roll(&self) -> bool {
        self.caps
            .as_ref()
            .zip(self.model.as_ref())
            .is_some_and(|(caps, model)| model.in_roll(caps))
    }

    pub fn roll_key(&self) -> Option<RollKey> {
        let model = self.model.as_ref()?;
        Some(RollKey {
            setup: model.device_setup(),
            invert: model.invert,
            probe: model.probes,
        })
    }

    pub fn roll_context(&self, session: u32) -> RollContext {
        let model = self.model.as_ref();
        let seconds_per_div = self.caps.as_ref().zip(model).and_then(|(caps, model)| {
            let step = caps.time_bases.get(usize::from(model.setup.time_base_index));
            step.map(|t| t.seconds_per_div)
        });
        RollContext {
            session,
            invert: model.map_or([false; CHANNELS], |m| m.invert),
            probe: model.map_or([1.0; CHANNELS], |m| m.probes),
            seconds_per_div: seconds_per_div.unwrap_or(0.0),
        }
    }

    pub fn set_channel(&mut self, channel: u8, patch: ChannelPatch) -> Result<Change, KernelError> {
        let number = channel_index(channel)?;
        validate_patch(&patch)?;
        self.edit(&move |caps, model| apply_channel_patch(caps, model, number, &patch))
    }

    pub fn step(&mut self, target: SettingTarget, direction: i8) -> Result<Change, KernelError> {
        let direction = direction.signum();
        if let SettingTarget::Range { channel } = target {
            channel_index(channel)?;
        }
        self.edit(&move |caps, model| {
            match target {
                SettingTarget::Range { channel } => step_range(caps, model, usize::from(channel) - 1, direction),
                SettingTarget::TimeBase => step_time_base(caps, model, direction),
                SettingTarget::TriggerPosition => {
                    model.trigger_fraction =
                        rounded(model.trigger_fraction + f64::from(direction) * POSITION_STEP).clamp(0.0, 1.0);
                }
                SettingTarget::TriggerLevel => {
                    let divisions = model.level_divisions(caps) + f64::from(direction) * LEVEL_STEP_DIVS;
                    set_level(caps, model, divisions);
                }
                SettingTarget::MathScale | SettingTarget::MathPosition => math::step(model, target, direction),
            }
            Ok(())
        })
    }

    pub fn reset(&mut self, target: SettingTarget) -> Result<Change, KernelError> {
        if let SettingTarget::Range { channel } = target {
            channel_index(channel)?;
        }
        let defaults = self.defaults.clone();
        self.edit(&move |caps, model| {
            match target {
                SettingTarget::Range { channel } => {
                    let index = nearest_index(&caps.vertical_ranges, defaults.range_volts_per_div, |r| r.volts_per_div);
                    set_range(caps, model, usize::from(channel) - 1, index);
                }
                SettingTarget::TimeBase => {
                    model.setup.time_base_index =
                        nearest_index(&caps.time_bases, defaults.time_base_seconds_per_div, |t| {
                            t.seconds_per_div
                        });
                }
                SettingTarget::TriggerPosition => model.trigger_fraction = defaults.trigger_fraction,
                SettingTarget::TriggerLevel => model.setup.trigger.level_volts = 0.0,
                SettingTarget::MathScale | SettingTarget::MathPosition => math::reset(model, target),
            }
            Ok(())
        })
    }

    pub fn set_trigger_position(&mut self, fraction: f64) -> Result<Change, KernelError> {
        ensure_finite(fraction)?;
        self.edit(&move |_, model| {
            model.trigger_fraction = fraction.clamp(0.0, 1.0);
            Ok(())
        })
    }

    pub fn set_trigger_level(&mut self, divisions: f64) -> Result<Change, KernelError> {
        ensure_finite(divisions)?;
        self.edit(&move |caps, model| {
            set_level(caps, model, divisions);
            Ok(())
        })
    }

    pub fn set_trigger(&mut self, patch: TriggerPatch) -> Result<Change, KernelError> {
        if let Some(source) = patch.source {
            channel_index(source)?;
        }
        self.edit(&move |_, model| {
            if let Some(source) = patch.source {
                let number = usize::from(source) - 1;
                if !model.setup.channels[number].enabled {
                    return Err(KernelError::recoverable(
                        "The trigger source must be an enabled channel",
                    ));
                }
                model.setup.trigger.source = number as u8;
            }
            model.setup.trigger.slope = patch.slope.unwrap_or(model.setup.trigger.slope);
            model.setup.trigger.sweep = patch.sweep.unwrap_or(model.setup.trigger.sweep);
            pulse::apply(model, &patch)
        })
    }

    /// `None` until the first device became ready; afterwards the last state, with controls unavailable while no device is ready.
    pub fn state(&self, revision: u64) -> Option<InstrumentState> {
        let (caps, model) = self.caps.as_ref().zip(self.model.as_ref())?;
        let locks = view::Locks {
            connection: self.unavailable.as_deref(),
            autoset_running: self.autoset_running,
        };
        let mut state = view::instrument_state(caps, model, revision, &locks);
        state.setup_notice.clone_from(&self.notice);
        Some(state)
    }

    pub fn facts(&self, revision: u64, meter: &CaptureMeter, now: Instant) -> Option<InstrumentFacts> {
        let (caps, model) = self.caps.as_ref().zip(self.model.as_ref())?;
        Some(view::instrument_facts(caps, model, meter, now, revision))
    }

    pub fn frame_context(&self, setup_revision: u64) -> FrameContext {
        let mut context = FrameContext {
            setup_revision,
            window_start: 0,
            window_len: 0,
            invert: [false; CHANNELS],
            probe: [1.0; CHANNELS],
            range_verified: [true; CHANNELS],
            seconds_per_div: 0.0,
        };
        let Some((caps, model)) = self.caps.as_ref().zip(self.model.as_ref()) else {
            return context;
        };
        let g = model.geometry(caps);
        context.window_start = g.window_start;
        context.window_len = g.window_len;
        context.invert = model.invert;
        context.probe = model.probes;
        context.seconds_per_div = caps
            .time_bases
            .get(usize::from(model.setup.time_base_index))
            .map_or(0.0, |t| t.seconds_per_div);
        for (number, verified) in context.range_verified.iter_mut().enumerate() {
            *verified = model.range(caps, number).is_none_or(|r| r.verified);
        }
        context
    }

    pub fn setup(&self) -> Option<&AcquisitionSetup> {
        self.model.as_ref().map(|m| &m.setup)
    }

    /// What the device must be given: the user's setup, with the sweep a Single needs.
    pub fn device_setup(&self) -> Option<AcquisitionSetup> {
        self.model.as_ref().map(Model::device_setup)
    }

    pub fn running(&self) -> bool {
        self.model.as_ref().is_some_and(|m| m.running)
    }

    /// Record duration of the current setup, for the dead time.
    pub fn record_duration_s(&self) -> Option<f64> {
        let (caps, model) = self.caps.as_ref().zip(self.model.as_ref())?;
        let interval = caps.sample_interval_s(model.enabled_count(), model.setup.time_base_index);
        Some(f64::from(caps.record_len) * interval)
    }

    fn edit(&mut self, change: Edit) -> Result<Change, KernelError> {
        if let Some(reason) = &self.unavailable {
            return Err(KernelError::recoverable(reason.clone()));
        }
        let (Some(caps), Some(model)) = (&self.caps, &mut self.model) else {
            return Err(KernelError::recoverable(NO_SCOPE));
        };
        let mut next = model.clone();
        change(caps, &mut next)?;
        next.settle(caps);
        let outcome = Change {
            setup_changed: next.setup != model.setup,
            state_changed: next != *model,
        };
        *model = next;
        Ok(outcome)
    }
}

fn channel_index(channel: u8) -> Result<usize, KernelError> {
    match usize::from(channel) {
        1..=CHANNELS => Ok(usize::from(channel) - 1),
        _ => Err(KernelError::recoverable("The channel must be 1 to 4")),
    }
}

fn ensure_finite(value: f64) -> Result<(), KernelError> {
    value
        .is_finite()
        .then_some(())
        .ok_or_else(|| KernelError::recoverable("The value must be a number"))
}

fn validate_patch(patch: &ChannelPatch) -> Result<(), KernelError> {
    if let Some(position) = patch.position_divs {
        ensure_finite(position)?;
    }
    match patch.probe {
        Some(probe) if !PROBES.contains(&probe) => Err(KernelError::recoverable("The probe must be 1x, 10x or 100x")),
        _ => Ok(()),
    }
}

fn apply_channel_patch(
    caps: &Capabilities,
    model: &mut Model,
    number: usize,
    patch: &ChannelPatch,
) -> Result<(), KernelError> {
    if let Some(enabled) = patch.enabled {
        model.setup.channels[number].enabled = enabled;
        if model.enabled_count() == 0 {
            return Err(KernelError::recoverable(LAST_CHANNEL));
        }
        model.keep_source_enabled();
    }
    let channel = &mut model.setup.channels[number];
    channel.coupling = patch.coupling.unwrap_or(channel.coupling);
    channel.bandwidth_limit = patch.bandwidth_limit.unwrap_or(channel.bandwidth_limit);
    channel.position_divs = patch.position_divs.unwrap_or(channel.position_divs);
    model.probes[number] = patch.probe.unwrap_or(model.probes[number]);
    model.invert[number] = patch.invert.unwrap_or(model.invert[number]);
    model.clamp_position(caps, number);
    Ok(())
}

fn set_range(caps: &Capabilities, model: &mut Model, number: usize, index: u8) {
    model.setup.channels[number].range_index = index;
    model.clamp_position(caps, number);
}

fn step_range(caps: &Capabilities, model: &mut Model, number: usize, direction: i8) {
    let last = caps.vertical_ranges.len().saturating_sub(1);
    let current = usize::from(model.setup.channels[number].range_index);
    let next = current.saturating_add_signed(isize::from(direction)).min(last);
    set_range(caps, model, number, next as u8);
}

/// Steps stop at the ends; the slow steps are roll mode.
fn step_time_base(caps: &Capabilities, model: &mut Model, direction: i8) {
    let current = usize::from(model.setup.time_base_index);
    let next = current.saturating_add_signed(isize::from(direction));
    if next < caps.time_bases.len() {
        model.setup.time_base_index = next as u8;
    }
}

fn set_level(caps: &Capabilities, model: &mut Model, divisions: f64) {
    model.set_level_divisions(caps, divisions.clamp(-SCREEN_HALF_HEIGHT_DIVS, SCREEN_HALF_HEIGHT_DIVS));
}

fn rounded(value: f64) -> f64 {
    (value * 1e6).round() / 1e6
}
