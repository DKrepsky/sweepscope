import { describe, expect, it } from 'vitest';
import { fitScreen, MIN_SCREEN } from './fit-screen';

describe('fitScreen', () => {
  it.each([
    [1000, 500, 625, 500],
    [2000, 800, 1000, 800],
    [800, 2000, 800, 640],
    [1000, 800, 1000, 800],
    [3840, 1600, 2000, 1600],
  ])('given a stage %i x %i, when fitted, then the screen is %i x %i', (w, h, width, height) => {
    expect(fitScreen({ width: w, height: h })).toEqual({ width, height });
  });

  it.each([
    [500, 1000],
    [400, 300],
    [2000, 300],
    [511, 511],
  ])('given a stage %i x %i below the minimum, when fitted, then the screen is the minimum 512 x 410', (w, h) => {
    expect(fitScreen({ width: w, height: h })).toEqual(MIN_SCREEN);
    expect(MIN_SCREEN).toEqual({ width: 512, height: 410 });
  });

  it('given a stage whose height is just under the minimum height, when fitted, then the screen is the minimum', () => {
    expect(fitScreen({ width: 512, height: 409 })).toEqual(MIN_SCREEN);
  });
});
