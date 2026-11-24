use std::sync::{Arc, Weak};
use std::time::Instant;

use sweepscope_kernel::{Bus, KernelError, Request};

use super::core::Core;
use super::effects::Effects;
use super::setup_files::SetupFiles;
use super::shared::Shared;
use crate::{
    ArmSingle, ForceTriggerNow, GetInstrument, LoadSetup, ResetSetting, SaveSetup, SetChannel, SetMath, SetRunning,
    SetTrigger, SetTriggerLevel, SetTriggerPosition, StartAutoset, StepSetting,
};

/// Registers every request of this feature. The handlers hold the module state weakly: the bus lives as long as the
/// application, the module may not.
pub fn register(bus: &Bus, shared: &Arc<Shared>, files: SetupFiles) -> Result<(), KernelError> {
    let state = Arc::downgrade(shared);
    let weak = &state;
    on(bus, weak, |core, r: SetRunning, fx, now| {
        core.set_running(r.running, fx, now)
    })?;
    on(bus, weak, |core, _: ForceTriggerNow, fx, _| core.force_trigger(fx))?;
    on(bus, weak, |core, _: ArmSingle, fx, now| core.arm_single(fx, now))?;
    on(bus, weak, |core, _: StartAutoset, fx, _| core.start_autoset(fx))?;
    on(bus, weak, |core, r: SetChannel, fx, now| {
        core.edit(fx, now, |i| i.set_channel(r.channel, r.patch))
    })?;
    on(bus, weak, |core, r: StepSetting, fx, now| {
        core.edit(fx, now, |i| i.step(r.target, r.direction))
    })?;
    on(bus, weak, |core, r: ResetSetting, fx, now| {
        core.edit(fx, now, |i| i.reset(r.target))
    })?;
    on(bus, weak, |core, r: SetTriggerPosition, fx, now| {
        core.edit(fx, now, |i| i.set_trigger_position(r.fraction))
    })?;
    on(bus, weak, |core, r: SetTriggerLevel, fx, now| {
        core.edit(fx, now, |i| i.set_trigger_level(r.divisions))
    })?;
    on(bus, weak, |core, r: SetMath, fx, now| {
        core.edit(fx, now, |i| i.set_math(r.patch))
    })?;
    on(bus, weak, |core, r: SetTrigger, fx, now| {
        core.edit(fx, now, |i| i.set_trigger(r.patch))
    })?;
    register_files(bus, &state, files)?;
    let reader = state;
    bus.handle(move |_: GetInstrument| {
        let shared = reader.upgrade().ok_or_else(gone)?;
        Ok(shared.snapshot())
    })
}

fn register_files(bus: &Bus, shared: &Weak<Shared>, files: SetupFiles) -> Result<(), KernelError> {
    let files = Arc::new(files);
    let (saver, save_files) = (shared.clone(), Arc::clone(&files));
    bus.handle(move |_: SaveSetup| {
        let shared = saver.upgrade().ok_or_else(gone)?;
        save_files.save(&shared)
    })?;
    let loader = shared.clone();
    bus.handle(move |_: LoadSetup| {
        let shared = loader.upgrade().ok_or_else(gone)?;
        files.load(&shared)
    })
}

type Step<R> = fn(&mut Core, R, &mut Effects, Instant) -> Result<(), KernelError>;

fn on<R>(bus: &Bus, shared: &Weak<Shared>, step: Step<R>) -> Result<(), KernelError>
where
    R: Request<Response = ()>,
{
    let shared = shared.clone();
    bus.handle(move |request: R| {
        let shared = shared.upgrade().ok_or_else(gone)?;
        shared.act(|core, fx, now| step(core, request, fx, now))
    })
}

fn gone() -> KernelError {
    KernelError::fatal("the acquisition module has stopped")
}
