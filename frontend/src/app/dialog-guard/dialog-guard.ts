export const STOP_ACQUISITION_FIRST = 'Stop acquisition first';

/** Design 10.3: no modal dialog while acquisition runs; the app never stops acquisition by itself. */
export function modalBlockedReason(acquiring: boolean): string | null {
  return acquiring ? STOP_ACQUISITION_FIRST : null;
}
