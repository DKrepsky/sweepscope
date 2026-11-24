import {
  acquisitionStatus,
  connectionSnapshot,
  displayState,
  instrumentFacts,
  instrumentState,
} from '../instrument/test-data';
import type { ModelInputs } from './select-common';

/** A ready scope with every slice present and nothing special going on; a test overrides what it is about. */
export function modelInputs(patch: Partial<ModelInputs> = {}): ModelInputs {
  return {
    connection: connectionSnapshot(),
    instrument: instrumentState(),
    status: acquisitionStatus(),
    facts: instrumentFacts(),
    display: displayState(),
    measurements: undefined,
    frame: undefined,
    selected: 1,
    cursors: {},
    pinned: [],
    renderer: 'Canvas 2D',
    spectrum: undefined,
    spectrumFrame: undefined,
    recording: undefined,
    roll: undefined,
    view: 'yt',
    xyAxes: { x: 1, y: 2 },
    peak: { mode: 'auto', channel: 1, bin: 0 },
    persistenceEpoch: 0,
    dismissedRecordingFailure: 0,
    ...patch,
  };
}
