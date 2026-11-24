import { describe, expect, it } from 'vitest';
import { decodeFrame } from './decode-frame';
import { buildFrame } from './test-frame';

describe('decodeFrame', () => {
  it('given a frame, when decoded, then samples are views of the buffer without a copy', () => {
    const buffer = buildFrame({ channels: [1, 2], samplesPerChannel: 3 });
    const { channels } = decodeFrame(buffer);
    expect(channels.every(({ samples }) => samples.buffer === buffer)).toBe(true);
  });

  it('given channels 2 and 4, when decoded, then they come in ascending order with their own samples', () => {
    const { channels } = decodeFrame(buildFrame({ channels: [2, 4], samplesPerChannel: 2 }));
    expect(channels.map(({ channel, samples }) => [channel, Array.from(samples)])).toEqual([
      [2, [2, 2]],
      [4, [4, 4]],
    ]);
  });

  it('given CH1 and the math trace, when decoded, then the math trace is channel 5 after CH1', () => {
    const { channels } = decodeFrame(buildFrame({ channels: [1, 5], samplesPerChannel: 2 }));
    expect(channels.map(({ channel, samples }) => [channel, Array.from(samples)])).toEqual([
      [1, [1, 1]],
      [5, [5, 5]],
    ]);
  });

  it('given a trigger index of minus one, when decoded, then the trigger index is undefined', () => {
    expect(decodeFrame(buildFrame({ triggerIndex: -1 })).triggerIndex).toBeUndefined();
  });

  it('given an unknown magic, when decoded, then it is rejected', () => {
    const buffer = buildFrame();
    new DataView(buffer).setUint32(0, 0x32465353, true);
    expect(() => decodeFrame(buffer)).toThrow(/magic/);
  });

  it('given a buffer shorter than the header, when decoded, then it is rejected', () => {
    expect(() => decodeFrame(new ArrayBuffer(10))).toThrow(/header/);
  });

  it('given a payload that does not match the sample count, when decoded, then it is rejected', () => {
    const buffer = buildFrame({ samplesPerChannel: 4 });
    expect(() => decodeFrame(buffer.slice(0, buffer.byteLength - 4))).toThrow(/length/);
  });
});
