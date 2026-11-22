//! Device-neutral instrument interfaces: one opened instrument (`ScopeDevice`), what exists and when it
//! appears (`DeviceCatalog`), and the value types they exchange. No hardware driver types appear here.
#![forbid(unsafe_code)]

mod capabilities;
mod catalog;
mod device;
mod error;
mod frame;
mod identity;
mod setup;
pub mod simulator;

pub use capabilities::{Capabilities, TimeBaseStep, VerticalRange};
pub use catalog::{CatalogEvent, DeviceCatalog};
pub use device::ScopeDevice;
pub use error::DeviceError;
pub use frame::{ChannelData, Frame, TriggerOutcome};
pub use identity::{CalibrationStatus, DeviceId, DeviceIdentity};
pub use setup::{AcquisitionSetup, ChannelSetup, Coupling, Slope, Sweep, TriggerSetup};
pub use simulator::signal::SignalKind;
pub use simulator::{SimulatedCatalog, SimulatedDevice, SimulatorControl};
