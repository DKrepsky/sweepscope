use std::sync::{Arc, Weak};
use std::time::Instant;

use sweepscope_kernel::{Bus, KernelError, Request};

use super::core::Core;
use super::effects::Effects;
use super::shared::Shared;
use crate::{
    ForceTriggerNow, GetInstrument, ResetSetting, SetChannel, SetRunning, SetTrigger, SetTriggerLevel,
    SetTriggerPosition, StepSetting,
};

/// Registers every request of this feature. The handlers hold the module state weakly: the bus lives as long as the
/// application, the module may not.
pub fn register(bus: &Bus, shared: &Arc<Shared>) -> Result<(), KernelError> {
    let state = Arc::downgrade(shared);
    let weak = &state;
    on(bus, weak, |core, r: SetRunning, fx, now| {
        core.set_running(r.running, fx, now)
    })?;
    on(bus, weak, |core, _: ForceTriggerNow, fx, _| core.force_trigger(fx))?;
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
    on(bus, weak, |core, r: SetTrigger, fx, now| {
        core.edit(fx, now, |i| i.set_trigger(r.patch))
    })?;
    let reader = state;
    bus.handle(move |_: GetInstrument| {
        let shared = reader.upgrade().ok_or_else(gone)?;
        Ok(shared.snapshot())
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
