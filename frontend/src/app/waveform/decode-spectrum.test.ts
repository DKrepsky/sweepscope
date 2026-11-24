import { describe, expect, it } from 'vitest';
import { decodeSpectrum } from './decode-spectrum';
import { buildSpectrum } from './test-roll';

describe('decodeSpectrum', () => {
  it('given a spectrum of CH2 and M, when decoded, then the header fields match', () => {
    const decoded = decodeSpectrum(
      buildSpectrum({ sequence: 4, channels: [2, 5], bins: 6, binHz: 12.5, linear: true, window: 3, setupRevision: 9 }),
    );
    expect(decoded).toMatchObject({
      sequence: 4,
      mask: 0b10010,
      linear: true,
      window: 'blackman',
      bins: 6,
      binHz: 12.5,
      top: 10,
      unitsPerDiv: 10,
      setupRevision: 9,
    });
  });

  it('given two channels, when decoded, then each has its own levels, divisions and peak bin', () => {
    const { channels } = decodeSpectrum(buildSpectrum({ channels: [1, 3], bins: 4, peakBins: [2, undefined] }));
    expect(channels.map((c) => [c.channel, c.peakBin, Array.from(c.levels), Array.from(c.divisions)])).toEqual([
      [1, 2, [-1, -1, -1, -1], [1, 1, 1, 1]],
      [3, undefined, [-3, -3, -3, -3], [3, 3, 3, 3]],
    ]);
  });

  it('given a spectrum, when decoded, then the arrays are views of the buffer', () => {
    const buffer = buildSpectrum();
    expect(decodeSpectrum(buffer).channels[0]?.levels.buffer).toBe(buffer);
  });

  it('given an unknown magic, when decoded, then it is rejected', () => {
    const buffer = buildSpectrum();
    new DataView(buffer).setUint32(0, 1, true);
    expect(() => decodeSpectrum(buffer)).toThrow(/magic/);
  });

  it('given an unknown window code, when decoded, then it is rejected', () => {
    const buffer = buildSpectrum();
    new DataView(buffer).setUint8(26, 4);
    expect(() => decodeSpectrum(buffer)).toThrow(/window/);
  });

  it('given a buffer shorter than the header, when decoded, then it is rejected', () => {
    expect(() => decodeSpectrum(new ArrayBuffer(20))).toThrow(/header/);
  });

  it('given a payload that does not match the bin count, when decoded, then it is rejected', () => {
    const buffer = buildSpectrum({ bins: 4 });
    expect(() => decodeSpectrum(buffer.slice(0, buffer.byteLength - 4))).toThrow(/length/);
  });
});
