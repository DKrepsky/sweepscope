use std::sync::mpsc::Receiver;

use crate::{DeviceError, DeviceId, ScopeDevice};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogEvent {
    Attached(DeviceId),
    Detached(DeviceId),
}

/// What exists and when it appears or goes away. An instrument can only be opened after it was attached.
pub trait DeviceCatalog: Send {
    fn events(&self) -> Receiver<CatalogEvent>;
    fn open(&self, id: &DeviceId) -> Result<Box<dyn ScopeDevice>, DeviceError>;
}
