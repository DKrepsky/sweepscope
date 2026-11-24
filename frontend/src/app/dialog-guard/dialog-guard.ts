import type { CalibrationTagModel } from '../calibration';

export const STOP_ACQUISITION_FIRST = 'Stop acquisition first';

/** Design 10.3: no modal dialog while acquisition runs; the app never stops acquisition by itself. */
export function modalBlockedReason(acquiring: boolean): string | null {
  return acquiring ? STOP_ACQUISITION_FIRST : null;
}

/**
 * The calibration dialog is the one modal that opens while acquisition runs: it holds acquisition itself. Only a missing
 * device, an unready scope or a recording in progress keep it shut.
 */
export function calibrationBlockedReason(tag: CalibrationTagModel): string | null {
  return tag.state === 'unavailable' ? tag.reason : null;
}
