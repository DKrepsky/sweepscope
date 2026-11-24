import { describe, expect, it } from 'vitest';
import { divisionsFromFraction, divisionsFromXFraction, xFractionFromGround } from './screen-geometry';

describe('divisionsFromFraction', () => {
  it('given the top of the screen, when converted, then it is four divisions above the centre', () => {
    expect(divisionsFromFraction(1)).toBe(4);
  });

  it('given the middle of the screen, when converted, then it is zero', () => {
    expect(divisionsFromFraction(0.5)).toBe(0);
  });

  it('given the bottom of the screen, when converted, then it is four divisions below the centre', () => {
    expect(divisionsFromFraction(0)).toBe(-4);
  });

  it('given a fraction outside the screen, when converted, then it is clamped to the screen', () => {
    expect(divisionsFromFraction(1.4)).toBe(4);
    expect(divisionsFromFraction(-0.2)).toBe(-4);
  });
});

describe('X axis geometry', () => {
  it('given a ground one division above the centre, when placed on the X axis, then it is one tenth right of the centre', () => {
    expect(xFractionFromGround(0.5 + 1 / 8)).toBeCloseTo(0.6, 9);
  });

  it('given an X fraction, when committed, then the channel position is in tenths of the width', () => {
    expect(divisionsFromXFraction(0.6)).toBeCloseTo(1, 9);
  });

  it('given a fraction outside the screen, when committed, then it is clamped to five divisions', () => {
    expect(divisionsFromXFraction(1.4)).toBe(5);
    expect(divisionsFromXFraction(-1)).toBe(-5);
  });
});
