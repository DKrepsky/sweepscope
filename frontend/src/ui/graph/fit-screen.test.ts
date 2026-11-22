import { describe, expect, it } from 'vitest';
import { fitScreen, minScreen } from './fit-screen';

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
  ])(
    'given a stage %i x %i below the minimum, when fitted, then the screen is the minimum 32 x 25.6 rem (512 x 409.6 px)',
    (w, h) => {
      expect(fitScreen({ width: w, height: h })).toEqual(minScreen());
      expect(minScreen()).toEqual({ width: 512, height: 409.6 });
    },
  );

  it('given a stage whose height is just under the minimum height, when fitted, then the screen is the minimum', () => {
    expect(fitScreen({ width: 512, height: 409 })).toEqual(minScreen());
  });

  it.each([
    [20, { width: 640, height: 512 }],
    [32, { width: 1024, height: 819.2 }],
  ])('given a root font of %i px, when fitted into a small stage, then the minimum scales in rem', (root, size) => {
    expect(fitScreen({ width: 500, height: 400 }, root)).toEqual(size);
  });

  it('given a root font of 20 px and a stage that fits 600 x 480 px, when fitted, then the screen is the 640 x 512 minimum', () => {
    expect(fitScreen({ width: 600, height: 480 }, 20)).toEqual(minScreen(20));
  });
});
