import { SCREEN_VERTICAL_DIVISIONS } from './select-common';

/** Inverse of `fractionFromCentre`: a marker's 0 (bottom) to 1 (top) screen fraction to divisions from the centre. */
export const divisionsFromFraction = (fraction: number): number => {
  const clamped = Math.min(1, Math.max(0, fraction));
  return (clamped - 0.5) * SCREEN_VERTICAL_DIVISIONS;
};
