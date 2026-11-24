import { describe, expect, it } from 'vitest';
import { decodeRollItem } from './decode-roll';
import { buildRollItem } from './test-roll';

describe('decodeRollItem', () => {
  it('given a samples item with CH1 and M, when decoded, then the fields and channel-major samples match', () => {
    const item = decodeRollItem(
      buildRollItem({ sequence: 7, session: 3, channels: [1, 5], samples: [1, 2, 3], firstIndex: 40 }),
    );
    expect(item).toMatchObject({ sequence: 7, kind: 'samples', session: 3, mask: 0b10001, firstIndex: 40, count: 3 });
    expect(
      item.channels.map(({ channel, samples }) => [channel, Array.from(samples, (v) => Math.round(v * 100))]),
    ).toEqual([
      [1, [101, 201, 301]],
      [5, [105, 205, 305]],
    ]);
  });

  it('given a samples item, when decoded, then the samples are views of the buffer', () => {
    const buffer = buildRollItem({ channels: [2] });
    expect(decodeRollItem(buffer).channels[0]?.samples.buffer).toBe(buffer);
  });

  it('given a gap item, when decoded, then it carries after and missed and no samples', () => {
    const item = decodeRollItem(buildRollItem({ kind: 'gap', firstIndex: 99, missed: 12 }));
    expect(item).toMatchObject({ kind: 'gap', firstIndex: 99, missed: 12, count: 0, channels: [] });
  });

  it('given a closed item, when decoded, then it carries the reason', () => {
    const item = decodeRollItem(buildRollItem({ kind: 'closed', closeReason: 'setupChanged' }));
    expect(item).toMatchObject({ kind: 'closed', closeReason: 'setupChanged' });
  });

  it('given a samples item, when decoded, then it has no close reason', () => {
    expect(decodeRollItem(buildRollItem()).closeReason).toBeUndefined();
  });

  it('given an unknown magic, when decoded, then it is rejected', () => {
    const buffer = buildRollItem();
    new DataView(buffer).setUint32(0, 0x32465353, true);
    expect(() => decodeRollItem(buffer)).toThrow(/magic/);
  });

  it('given an unknown kind, when decoded, then it is rejected', () => {
    const buffer = buildRollItem();
    new DataView(buffer).setUint8(8, 9);
    expect(() => decodeRollItem(buffer)).toThrow(/kind/);
  });

  it('given a buffer shorter than the header, when decoded, then it is rejected', () => {
    expect(() => decodeRollItem(new ArrayBuffer(8))).toThrow(/header/);
  });

  it('given a payload that does not match the sample count, when decoded, then it is rejected', () => {
    const buffer = buildRollItem({ samples: [1, 2, 3] });
    expect(() => decodeRollItem(buffer.slice(0, buffer.byteLength - 4))).toThrow(/length/);
  });
});
