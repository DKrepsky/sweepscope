//! Hantek adapter (WP4 fills this in): the only code that names `hantek6xx4-lib`.

use std::marker::PhantomData;
use std::sync::mpsc::Receiver;

use hantek6xx4_lib::Transport;

use crate::{CatalogEvent, DeviceCatalog, DeviceError, DeviceId, ScopeDevice};

pub struct HantekCatalog(());

impl HantekCatalog {
    pub fn start() -> Result<Self, DeviceError> {
        Err(DeviceError::NotFound)
    }
}

impl DeviceCatalog for HantekCatalog {
    fn events(&self) -> Receiver<CatalogEvent> {
        unreachable!("start() never returns a catalog until the adapter exists")
    }

    fn open(&self, _id: &DeviceId) -> Result<Box<dyn ScopeDevice>, DeviceError> {
        Err(DeviceError::NotFound)
    }
}

pub struct HantekDevice<T: Transport>(PhantomData<T>);
