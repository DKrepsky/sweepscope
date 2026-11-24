//! Device-neutral instrument interfaces: one opened instrument (`ScopeDevice`), what exists and when it
//! appears (`DeviceCatalog`), and the session module that connects them to the bus. The value types live in
//! `sweepscope_kernel::contracts` and are re-exported here; driver types appear only in `hantek`.
#![forbid(unsafe_code)]

mod catalog;
mod device;
mod error;
mod hantek;
mod identity;
mod messages;
mod module;
mod profile;
mod simulator;

pub use catalog::{CatalogEvent, DeviceCatalog};
pub use device::ScopeDevice;
pub use error::DeviceError;
pub use hantek::{HantekCatalog, HantekDevice};
pub use identity::DeviceId;
pub use messages::{GetConnection, RetryConnection, UseSimulator};
pub use module::{DeviceDeps, DeviceModule, SimulatorSource, COMMANDS};
pub use simulator::{device_id, SignalKind, SimulatedCatalog, SimulatedDevice, SimulatorControl};
pub use sweepscope_kernel::contracts::{
    AcquisitionSetup, CalibrationStatus, Capabilities, ChannelData, ChannelSetup, Coupling, DeviceIdentity, Frame,
    Slope, Sweep, TimeBaseStep, TriggerKind, TriggerOutcome, TriggerSetup, VerticalRange,
};
