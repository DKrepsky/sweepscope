import { NOT_MEASURED, unavailableReason, type ModelInputs } from './select-common';
import type { PeakModel } from './types';

/** Placeholder until the peak panel work package: the marker state is shown, no number is known. */
export function selectPeak({ connection, peak }: ModelInputs): PeakModel {
  return {
    ...peak,
    frequency: NOT_MEASURED,
    level: NOT_MEASURED,
    available: { enabled: false, reason: unavailableReason(connection) },
  };
}
