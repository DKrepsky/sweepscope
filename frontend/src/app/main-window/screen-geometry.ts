import { SCREEN_VERTICAL_DIVISIONS } from './select-common';

/** Inverse of `fractionFromCentre`: a marker's 0 (bottom) to 1 (top) screen fraction to divisions from the centre. */
export const divisionsFromFraction = (fraction: number): number => {
  const clamped = Math.min(1, Math.max(0, fraction));
  return (clamped - 0.5) * SCREEN_VERTICAL_DIVISIONS;
};

const SCREEN_HORIZONTAL_DIVISIONS = 10;

/** XY: the X ground marker sits on the horizontal axis, where a division is a tenth of the width, not an eighth. */
export const xFractionFromGround = (groundPosition: number): number =>
  Math.min(1, Math.max(0, 0.5 + ((groundPosition - 0.5) * SCREEN_VERTICAL_DIVISIONS) / SCREEN_HORIZONTAL_DIVISIONS));

export const divisionsFromXFraction = (fraction: number): number =>
  (Math.min(1, Math.max(0, fraction)) - 0.5) * SCREEN_HORIZONTAL_DIVISIONS;
