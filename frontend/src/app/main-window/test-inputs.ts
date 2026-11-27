import {
  acquisitionStatus,
  connectionSnapshot,
  displayState,
  instrumentFacts,
  instrumentState,
  measurementsState,
  spectrumState,
  zeroCalibrationState,
} from '../instrument/test-data';
import type { ModelInputs } from './select-common';

/** A connected, running scope with nothing drawn yet; a test patches what it is about. */
export function modelInputs(patch: Partial<ModelInputs> = {}): ModelInputs {
  return {
    connection: connectionSnapshot(),
    instrument: instrumentState(),
    status: acquisitionStatus(),
    facts: instrumentFacts(),
    display: displayState(),
    measurements: measurementsState(),
    zeroCalibration: zeroCalibrationState(),
    frame: undefined,
    selected: 1,
    cursors: {},
    pinned: [],
    renderer: 'Canvas 2D',
    spectrum: spectrumState(),
    dismissedRecordingFailure: 0,
    pulseWidthRejected: false,
    spectrumFrame: undefined,
    recording: undefined,
    roll: undefined,
    view: 'yt',
    xyAxes: { x: 1, y: 2 },
    peak: { mode: 'auto', channel: 1, bin: 0 },
    persistenceEpoch: 0,
    ...patch,
  };
}
