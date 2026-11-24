import { recordingChip } from '../recording';
import { toAvailability, unavailableReason, type ModelInputs } from './select-common';
import type { RecordingModel } from './types';

export function selectRecording({ connection, instrument, recording }: ModelInputs): RecordingModel {
  const failure = recording?.failure;
  return {
    active: recording?.active ?? false,
    chip: recordingChip(recording),
    available: instrument
      ? toAvailability(instrument.controls.record)
      : { enabled: false, reason: unavailableReason(connection) },
    ...(failure ? { failure: { text: failure.text, fileName: failure.fileName } } : {}),
  };
}
