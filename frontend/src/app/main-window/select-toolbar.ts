import type { AcquisitionStatus, ConnectionSnapshot, InstrumentState } from '../../api';
import { isAcquiring, statusChips } from '../instrument';
import { toAvailability, unavailableReason } from './select-common';
import type { FilesAvailability, ToolbarModel } from './types';

/** MOCK: the toolbar has no backend state for persistence, view or tools yet (later stories). */
const PLACEHOLDERS = {
  persistence: 'Persist: Off',
  view: 'yt',
  tools: { math: false, reference: false },
} as const;

const NO_WAVEFORM = { enabled: false, reason: 'No waveform' } as const;

/** Setups follow the backend; exports and the screenshot need a frame to write. */
function selectFiles(
  connection: ConnectionSnapshot | undefined,
  instrument: InstrumentState | undefined,
  hasWaveform: boolean,
): FilesAvailability {
  const setups = instrument
    ? toAvailability(instrument.controls.files)
    : { enabled: false, reason: unavailableReason(connection) };
  const frame = hasWaveform ? { enabled: true } : NO_WAVEFORM;
  return { saveSetup: setups, loadSetup: setups, exportCsv: frame, exportRaw: frame, screenshot: frame };
}

export interface ToolbarExtras {
  hasWaveform: boolean;
  cursorsOn: boolean;
}

export function selectToolbar(
  connection: ConnectionSnapshot | undefined,
  instrument: InstrumentState | undefined,
  status: AcquisitionStatus | undefined,
  { hasWaveform, cursorsOn }: ToolbarExtras,
): ToolbarModel {
  const unavailable = { enabled: false, reason: unavailableReason(connection) };
  return {
    running: isAcquiring(connection, instrument),
    chips: statusChips(connection, instrument, status),
    ...PLACEHOLDERS,
    tools: { ...PLACEHOLDERS.tools, cursors: cursorsOn },
    cursorsAvailability: hasWaveform || cursorsOn ? { enabled: true } : NO_WAVEFORM,
    files: selectFiles(connection, instrument, hasWaveform),
    availability: instrument ? toAvailability(instrument.controls.run) : unavailable,
    force: instrument ? toAvailability(instrument.controls.force) : unavailable,
  };
}
