import type { AcquisitionStatus, ConnectionSnapshot, InstrumentState } from '../../api';
import { isAcquiring, statusChips } from '../instrument';
import { toAvailability, unavailableReason } from './select-common';
import type { ToolbarModel } from './types';

/** MOCK: the toolbar has no backend state for persistence, view or tools yet (later stories). */
const PLACEHOLDERS = {
  persistence: 'Persist: Off',
  view: 'yt',
  tools: { cursors: false, math: false, reference: false },
} as const;

export function selectToolbar(
  connection: ConnectionSnapshot | undefined,
  instrument: InstrumentState | undefined,
  status: AcquisitionStatus | undefined,
): ToolbarModel {
  const unavailable = { enabled: false, reason: unavailableReason(connection) };
  return {
    running: isAcquiring(connection, instrument),
    chips: statusChips(connection, instrument, status),
    ...PLACEHOLDERS,
    availability: instrument ? toAvailability(instrument.controls.run) : unavailable,
    force: instrument ? toAvailability(instrument.controls.force) : unavailable,
  };
}
