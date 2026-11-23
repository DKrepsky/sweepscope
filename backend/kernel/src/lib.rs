//! Shared kernel: message bus, clock, storage and the common error type.
#![forbid(unsafe_code)]

mod bus;
mod clock;
mod command;
pub mod contracts;
mod error;
mod files;
mod storage;
pub mod testing;

pub use bus::{Bus, Event, Request, Subscription};
pub use clock::{Clock, SystemClock};
pub use command::CommandDescriptor;
pub use error::{ErrorCategory, KernelError};
pub use files::{read_bounded, write_atomic, FileFilter, FilePicker};
pub use storage::{FileStorage, Storage};
