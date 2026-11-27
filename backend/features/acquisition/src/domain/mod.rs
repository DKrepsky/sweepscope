//! Pure instrument rules: geometry, `Instrument`, `next_status`, `CaptureMeter`, defaults (ADR 0010, ADR 0013).

pub(crate) mod autoset_run;
mod defaults;
pub mod geometry;
mod instrument;
mod math;
mod meter;
mod model;
mod offsets;
mod pulse;
pub mod setup_file;
mod status;
mod view;

pub use defaults::Defaults;
pub use geometry::{geometry, record_percent, trigger_index_in_window, Geometry};
pub use instrument::{Change, Instrument, Restore, RollKey, SetupReport};
pub use meter::CaptureMeter;
pub use setup_file::SetupFile;
pub use status::{next_status, StatusEvent};
