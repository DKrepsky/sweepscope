/** Builders for ADR 0011 wire frames, so e2e tests can drive the live graph without a backend. */
const HEADER = 48;

/** One ADR 0011 frame: channels in `mask` (bit 0 = CH1), `count` samples, every channel a ramp or a glitch. */
export function wireFrame(options: {
  mask: number;
  count: number;
  flags?: number;
  value: (channel: number, i: number) => number;
}) {
  const channels = [0, 1, 2, 3].filter((bit) => options.mask & (1 << bit));
  const bytes = new ArrayBuffer(HEADER + 4 * options.count * channels.length);
  const view = new DataView(bytes);
  view.setUint32(0, 0x31465353, true);
  view.setUint8(24, options.mask);
  view.setUint8(25, options.flags ?? 1);
  view.setUint32(28, options.count, true);
  view.setFloat64(32, 4e-8, true);
  view.setInt32(40, Math.floor(options.count / 2), true);
  view.setUint32(44, 1, true);
  channels.forEach((bit, slot) => {
    for (let i = 0; i < options.count; i += 1) {
      view.setFloat32(HEADER + 4 * (slot * options.count + i), options.value(bit + 1, i), true);
    }
  });
  return Array.from(new Uint8Array(bytes));
}

/** Three periods of a sine, one phase per channel, in divisions from the top edge. */
export const sine = (channel: number, i: number): number => 4 + 2.5 * Math.sin((i / 250) * Math.PI * 2 + channel);
