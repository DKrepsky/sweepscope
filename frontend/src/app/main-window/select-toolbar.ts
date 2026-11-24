import type { ConnectionSnapshot, InstrumentState } from '../../api';
import { isAcquiring, statusChips } from '../instrument';
import { toAvailability, unavailableReason, type ModelInputs } from './select-common';
import type { Availability, FilesAvailability, ToolbarModel } from './types';

/** MOCK: no backend state for a reference waveform yet (out of scope for M3). */
const REFERENCE_OFF = false;

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

/** With sweep Auto only a Single waits for a trigger; a Normal sweep waits too, so the pressed look is certain in Auto only. */
function singleArmed({ instrument, status }: ModelInputs): boolean {
  return instrument?.running === true && instrument.trigger.sweep === 'auto' && status?.phase === 'waitingForTrigger';
}

function controlAvailability(
  { connection, instrument }: ModelInputs,
  pick: (controls: InstrumentState['controls']) => InstrumentState['controls']['run'],
): Availability {
  return instrument
    ? toAvailability(pick(instrument.controls))
    : { enabled: false, reason: unavailableReason(connection) };
}

export function selectToolbar(inputs: ModelInputs, { hasWaveform, cursorsOn }: ToolbarExtras): ToolbarModel {
  const { connection, instrument, status } = inputs;
  return {
    running: isAcquiring(connection, instrument),
    chips: statusChips(connection, instrument, status),
    single: { armed: singleArmed(inputs), availability: controlAvailability(inputs, (c) => c.single) },
    tools: { cursors: cursorsOn, math: instrument?.math.enabled ?? false, reference: REFERENCE_OFF },
    mathAvailability: controlAvailability(inputs, (c) => c.frameViews),
    cursorsAvailability: hasWaveform || cursorsOn ? { enabled: true } : NO_WAVEFORM,
    files: selectFiles(connection, instrument, hasWaveform),
    availability: controlAvailability(inputs, (c) => c.run),
    force: controlAvailability(inputs, (c) => c.force),
  };
}
