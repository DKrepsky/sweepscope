#![allow(dead_code)]
use std::sync::Arc;
use std::time::Instant;

use sweepscope_device::{
    AcquisitionSetup, ChannelSetup, Coupling, ScopeDevice, SimulatedCatalog, SimulatedDevice, SimulatorControl, Slope,
    Sweep, TriggerSetup,
};
use sweepscope_kernel::testing::FakeClock;
use sweepscope_kernel::Clock;

pub const TB_2NS: u8 = 0;
pub const TB_200US: u8 = 15;
pub const TB_200MS: u8 = 24;
pub const TB_500MS: u8 = 25;
pub const RANGE_1V: u8 = 8;

pub struct Rig {
    pub clock: FakeClock,
    pub control: SimulatorControl,
    pub device: SimulatedDevice,
}

pub fn rig() -> Rig {
    let clock = FakeClock::default();
    let control = SimulatorControl::new();
    let device = SimulatedDevice::new(Arc::new(clock.clone()), control.clone());
    Rig { clock, control, device }
}

pub fn catalog(clock: &FakeClock, control: &SimulatorControl) -> SimulatedCatalog {
    SimulatedCatalog::new(Arc::new(clock.clone()), control.clone())
}

pub fn channel(enabled: bool) -> ChannelSetup {
    ChannelSetup {
        enabled,
        range_index: RANGE_1V,
        position_divs: 0.0,
        coupling: Coupling::Dc,
        bandwidth_limit: false,
    }
}

pub fn setup(enabled: [bool; 4], time_base_index: u8) -> AcquisitionSetup {
    AcquisitionSetup {
        channels: enabled.map(channel),
        time_base_index,
        trigger: TriggerSetup {
            source: 0,
            level_volts: 0.0,
            slope: Slope::Rising,
            sweep: Sweep::Auto,
            horizontal_position_percent: 50.0,
        },
    }
}

pub fn configured(rig: &mut Rig, setup: &AcquisitionSetup) {
    rig.device.configure(setup).unwrap();
}

impl Rig {
    pub fn clock_now(&self) -> Instant {
        self.clock.now()
    }
}
