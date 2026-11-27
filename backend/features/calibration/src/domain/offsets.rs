//! The offsets file: several units side by side, versioned, replaced atomically through `Storage`.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sweepscope_kernel::contracts::ZeroOffsets;
use sweepscope_kernel::{KernelError, Storage};

pub const FILE_KEY: &str = "zero-offsets.json";
pub const CORRUPT_KEY: &str = "zero-offsets.json.corrupt";
const FORMAT: &str = "sweepscope-zero-offsets";
const VERSION: u32 = 1;
/// Four times the largest range (10 V/div): a stored offset beyond it is damage, not calibration.
const MAX_OFFSET_VOLTS: f64 = 40.0;

/// Offsets of one unit: volts at the input, per channel and range index.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredUnit {
    pub stored_at_utc: String,
    pub volts: [Vec<f64>; 4],
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
struct Body {
    format: String,
    version: u32,
    units: BTreeMap<String, StoredUnit>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct OffsetFile {
    units: BTreeMap<String, StoredUnit>,
}

impl OffsetFile {
    /// A missing file is empty. An unreadable or foreign file is kept once as a `.corrupt` copy and treated as empty.
    pub fn load(storage: &dyn Storage) -> Result<Self, KernelError> {
        let Some(bytes) = storage.read(FILE_KEY)? else {
            return Ok(Self::default());
        };
        match parse(&bytes) {
            Some(units) => Ok(Self { units }),
            None => {
                keep_corrupt_copy(storage, &bytes)?;
                Ok(Self::default())
            }
        }
    }

    pub fn unit(&self, unit_id: &str) -> Option<&StoredUnit> {
        self.units.get(unit_id)
    }

    pub fn offsets_for(&self, unit_id: &str) -> Option<ZeroOffsets> {
        self.unit(unit_id).map(|unit| ZeroOffsets {
            unit_id: unit_id.to_owned(),
            volts: unit.volts.clone(),
        })
    }

    pub fn with_unit(mut self, unit_id: &str, unit: StoredUnit) -> Self {
        self.units.insert(unit_id.to_owned(), unit);
        self
    }

    pub fn save(&self, storage: &dyn Storage) -> Result<(), KernelError> {
        let body = Body {
            format: FORMAT.to_owned(),
            version: VERSION,
            units: self.units.clone(),
        };
        let bytes = serde_json::to_vec_pretty(&body)
            .map_err(|error| KernelError::recoverable(format!("cannot encode the zero offsets: {error}")))?;
        storage.write(FILE_KEY, &bytes)
    }
}

fn parse(bytes: &[u8]) -> Option<BTreeMap<String, StoredUnit>> {
    let body: Body = serde_json::from_slice(bytes).ok()?;
    let sane = body.units.values().all(|unit| {
        unit.volts
            .iter()
            .flatten()
            .all(|v| v.is_finite() && v.abs() <= MAX_OFFSET_VOLTS)
    });
    (body.format == FORMAT && body.version == VERSION && sane).then_some(body.units)
}

fn keep_corrupt_copy(storage: &dyn Storage, bytes: &[u8]) -> Result<(), KernelError> {
    if storage.read(CORRUPT_KEY)?.is_none() {
        storage.write(CORRUPT_KEY, bytes)?;
    }
    Ok(())
}
