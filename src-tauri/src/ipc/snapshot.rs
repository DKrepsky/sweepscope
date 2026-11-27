use serde::Serialize;
use sweepscope_acquisition::{AcquisitionStatus, GetInstrument, InstrumentFacts, InstrumentSnapshot, InstrumentState};
use sweepscope_calibration::{GetZeroCalibration, ZeroCalibrationState};
use sweepscope_device::GetConnection;
use sweepscope_display::{DisplayState, GetDisplay};
use sweepscope_kernel::contracts::ConnectionSnapshot;
use sweepscope_kernel::{Bus, KernelError};
use sweepscope_measurement::{GetMeasurements, MeasurementsState};
use sweepscope_recording::{GetRecording, RecordingState};
use sweepscope_spectrum::{GetSpectrum, SpectrumState};

/// What `get_snapshot` returns; the shell assembles it from the slice owners and holds no state of its own.
/// Its TypeScript type is rendered by `manifest::render_state_snapshot_ts`, because ts-rs cannot place a file of the
/// shell next to the feature types and still import them (a test compares the field names and types).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StateSnapshot {
    pub connection: ConnectionSnapshot,
    pub instrument: Option<InstrumentState>,
    pub status: AcquisitionStatus,
    pub facts: InstrumentFacts,
    pub display: DisplayState,
    pub measurements: MeasurementsState,
    pub spectrum: SpectrumState,
    pub recording: RecordingState,
    pub zero_calibration: ZeroCalibrationState,
}

pub fn snapshot_with(bus: &Bus) -> Result<StateSnapshot, KernelError> {
    let connection = bus.request(GetConnection)?;
    let InstrumentSnapshot {
        instrument,
        status,
        facts,
    } = bus.request(GetInstrument)?;
    Ok(StateSnapshot {
        connection,
        instrument,
        status,
        facts,
        display: bus.request(GetDisplay)?,
        measurements: bus.request(GetMeasurements)?,
        spectrum: bus.request(GetSpectrum)?,
        recording: bus.request(GetRecording)?,
        zero_calibration: bus.request(GetZeroCalibration)?,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use sweepscope_display::InterpolationMode;
    use sweepscope_kernel::contracts::ConnectionState;
    use sweepscope_kernel::testing::{eventually, FakeClock, MemoryStorage, ScriptedPicker};

    use super::*;
    use crate::composition::{compose, simulator_source, unix_time_us, Wiring};
    use crate::manifest::SNAPSHOT_FIELDS;

    fn compose_quiet() -> crate::composition::Backend {
        let clock = Arc::new(FakeClock::default());
        compose(Wiring {
            storage: Arc::new(MemoryStorage::default()),
            picker: Arc::new(ScriptedPicker::default()),
            record_files: Arc::new(sweepscope_recording::OsRecordFiles),
            clock: clock.clone(),
            hardware: None,
            simulator: Some(simulator_source(clock, true)),
            unix_time_us,
        })
        .expect("compose")
    }

    #[test]
    fn given_no_slice_handlers_when_a_snapshot_is_requested_then_the_error_has_category_and_message_without_paths() {
        let error = snapshot_with(&Bus::new()).expect_err("no handler registered");

        let json = serde_json::to_value(&error).expect("serialise");
        assert!(json["category"].is_string());
        assert!(json["message"].is_string());
        assert!(!error.message.contains('/') && !error.message.contains('\\'));
    }

    #[test]
    fn given_a_snapshot_when_serialised_then_its_fields_are_those_of_the_snapshot_manifest() {
        let backend = compose_quiet();

        let value = serde_json::to_value(snapshot_with(&backend.bus).expect("snapshot")).expect("serialise");

        let mut serialised: Vec<&str> = value.as_object().expect("object").keys().map(String::as_str).collect();
        let mut listed: Vec<&str> = SNAPSHOT_FIELDS.iter().map(|(name, _, _)| *name).collect();
        serialised.sort_unstable();
        listed.sort_unstable();
        assert_eq!(serialised, listed);
    }

    fn short_name(path: &str) -> &str {
        path.rsplit("::").next().expect("type name")
    }

    /// The TypeScript expression of a Rust field type: `Option<T>` is `T | null`, a struct keeps its name.
    fn ts_expression<T>(_: &T) -> String {
        let name = std::any::type_name::<T>();
        match name
            .strip_prefix("core::option::Option<")
            .and_then(|n| n.strip_suffix('>'))
        {
            Some(inner) => format!("{} | null", short_name(inner)),
            None => short_name(name).to_owned(),
        }
    }

    #[test]
    fn given_a_snapshot_when_its_field_types_are_read_then_they_are_those_of_the_snapshot_manifest() {
        let backend = compose_quiet();
        let StateSnapshot {
            connection,
            instrument,
            status,
            facts,
            display,
            measurements,
            spectrum,
            recording,
            zero_calibration,
        } = snapshot_with(&backend.bus).expect("snapshot");

        let actual = [
            ("connection", ts_expression(&connection)),
            ("instrument", ts_expression(&instrument)),
            ("status", ts_expression(&status)),
            ("facts", ts_expression(&facts)),
            ("display", ts_expression(&display)),
            ("measurements", ts_expression(&measurements)),
            ("spectrum", ts_expression(&spectrum)),
            ("recording", ts_expression(&recording)),
            ("zeroCalibration", ts_expression(&zero_calibration)),
        ];

        let listed: Vec<(&str, String)> = SNAPSHOT_FIELDS
            .iter()
            .map(|(name, ty, _)| (*name, (*ty).to_owned()))
            .collect();
        assert_eq!(listed, actual);
    }

    #[test]
    fn given_the_composed_app_with_the_simulator_when_a_snapshot_is_requested_then_it_carries_all_nine_slices() {
        let backend = compose_quiet();
        assert!(eventually(Duration::from_secs(5), || {
            snapshot_with(&backend.bus).is_ok_and(|s| s.connection.state == ConnectionState::Ready)
        }));

        let snapshot = snapshot_with(&backend.bus).expect("snapshot");

        assert!(snapshot.connection.simulator_active);
        assert!(snapshot.display.automatic);
        assert_eq!(snapshot.display.interpolation, InterpolationMode::Linear);
        assert!(snapshot.measurements.channels.is_empty());
        assert!(!snapshot.spectrum.active);
        assert!(!snapshot.recording.active);
        assert_eq!(snapshot.zero_calibration.session, None);
    }
}
