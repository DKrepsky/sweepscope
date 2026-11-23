//! Pure instrument rules: geometry, `Instrument`, `next_status`, `CaptureMeter`, defaults (ADR 0010, ADR 0013).

mod defaults;
pub mod geometry;
mod instrument;
mod meter;
mod model;
mod status;
mod view;

pub use defaults::Defaults;
pub use geometry::{geometry, record_percent, trigger_index_in_window, Geometry};
pub use instrument::{Change, Instrument, Restore};
pub use meter::CaptureMeter;
pub use status::{next_status, StatusEvent};
