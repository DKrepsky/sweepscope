import { describe, expect, it } from 'vitest';
import { divisionsFromFraction } from './screen-geometry';

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
