//! Shared kernel: message bus, clock, storage and the common error type.
#![forbid(unsafe_code)]

mod bus;
mod clock;
mod command;
mod error;
mod storage;
pub mod testing;

pub use bus::{Bus, Event, Request, Subscription};
pub use clock::{Clock, SystemClock};
pub use command::CommandDescriptor;
pub use error::{ErrorCategory, KernelError};
pub use storage::{FileStorage, Storage};
