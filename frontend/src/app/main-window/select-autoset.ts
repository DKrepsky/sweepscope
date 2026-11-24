import { toAvailability, unavailableReason, type ModelInputs } from './select-common';
import type { AutosetModel } from './types';

/** Placeholder until the toolbar work package: availability follows the backend, nothing is busy. */
export function selectAutoset({ connection, instrument, status }: ModelInputs): AutosetModel {
  return {
    busy: status?.autoset.running ?? false,
    noSignal: status?.autoset.noSignal ?? false,
    available: instrument
      ? toAvailability(instrument.controls.autoset)
      : { enabled: false, reason: unavailableReason(connection) },
  };
}
