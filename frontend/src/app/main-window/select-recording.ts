import { toAvailability, unavailableReason, type ModelInputs } from './select-common';
import type { RecordingModel } from './types';

/** Placeholder until the toolbar work package: availability and the active flag follow the backend, no chip text. */
export function selectRecording({ connection, instrument, recording }: ModelInputs): RecordingModel {
  return {
    active: recording?.active ?? false,
    available: instrument
      ? toAvailability(instrument.controls.record)
      : { enabled: false, reason: unavailableReason(connection) },
  };
}
