//! Hantek adapter: the only code that names `hantek6xx4-lib`.

mod catalog;
mod convert;
mod device;
#[cfg(test)]
mod device_tests;

pub use catalog::HantekCatalog;
pub use device::HantekDevice;
