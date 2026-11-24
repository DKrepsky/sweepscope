import { toAvailability, unavailableReason, type ModelInputs } from './select-common';
import type { AutosetModel, Availability } from './types';

const RUNNING = 'Auto-setup is running';

/** Not available while a run is in progress: the run cannot be cancelled and a second press would queue another. */
function availability({ connection, instrument }: ModelInputs, busy: boolean): Availability {
  if (!instrument) return { enabled: false, reason: unavailableReason(connection) };
  if (busy) return { enabled: false, reason: RUNNING };
  return toAvailability(instrument.controls.autoset);
}

export function selectAutoset(inputs: ModelInputs): AutosetModel {
  const busy = inputs.status?.autoset.running ?? false;
  return {
    busy,
    noSignal: inputs.status?.autoset.noSignal ?? false,
    available: availability(inputs, busy),
  };
}
