import { channelsInMask, type TraceChannel } from './trace-channel';
import type { RollCloseReason, RollItemKind } from './decode-roll';

const maskOf = (channels: TraceChannel[]) => channels.reduce((bits, channel) => bits | (1 << (channel - 1)), 0);

export interface TestRollItem {
  sequence?: number;
  kind?: RollItemKind;
  session?: number;
  channels?: TraceChannel[];
  /** Samples of every channel; the channel number / 100 is added so a test can tell channels apart. */
  samples?: number[];
  firstIndex?: number;
  missed?: number;
  closeReason?: RollCloseReason;
}

const KINDS: RollItemKind[] = ['samples', 'gap', 'closed'];
const CLOSE_REASONS: RollCloseReason[] = ['stopped', 'leftRoll', 'disconnected', 'setupChanged'];

const ROLL_DEFAULTS: Required<TestRollItem> = {
  sequence: 1,
  kind: 'samples',
  session: 1,
  channels: [1],
  samples: [0, 0],
  firstIndex: 0,
  missed: 0,
  closeReason: 'stopped',
};

/** Builds an ADR 0017 item. */
export function buildRollItem(options: TestRollItem = {}): ArrayBuffer {
  const { sequence, kind, session, channels, firstIndex, missed, samples, closeReason } = {
    ...ROLL_DEFAULTS,
    ...options,
  };
  const present = kind === 'samples' ? channelsInMask(maskOf(channels)) : [];
  const count = present.length > 0 ? samples.length : 0;
  const buffer = new ArrayBuffer(32 + 4 * count * present.length);
  const view = new DataView(buffer);
  view.setUint32(0, 0x314c5353, true);
  view.setUint32(4, sequence, true);
  view.setUint8(8, KINDS.indexOf(kind));
  view.setUint8(9, maskOf(channels));
  view.setUint8(10, CLOSE_REASONS.indexOf(closeReason));
  view.setUint32(12, count, true);
  view.setBigUint64(16, BigInt(firstIndex), true);
  view.setUint32(24, missed, true);
  view.setUint32(28, session, true);
  present.forEach((channel, slot) => {
    samples.slice(0, count).forEach((value, i) => {
      view.setFloat32(32 + 4 * (slot * count + i), value + channel / 100, true);
    });
  });
  return buffer;
}

export interface TestSpectrum {
  sequence?: number;
  channels?: TraceChannel[];
  bins?: number;
  binHz?: number;
  linear?: boolean;
  window?: number;
  peakBins?: (number | undefined)[];
  setupRevision?: number;
}

const SPECTRUM_DEFAULTS: Required<TestSpectrum> = {
  sequence: 1,
  channels: [1],
  bins: 5,
  binHz: 12.5,
  linear: false,
  window: 1,
  peakBins: [],
  setupRevision: 1,
};

const NO_PEAK = 0xffffffff;

/** Builds an ADR 0018 frame; channel `c` has level `-c` and divisions `c` in every bin. */
export function buildSpectrum(options: TestSpectrum = {}): ArrayBuffer {
  const { sequence, channels, bins, binHz, peakBins, linear, window, setupRevision } = {
    ...SPECTRUM_DEFAULTS,
    ...options,
  };
  const buffer = new ArrayBuffer(56 + 4 * channels.length + 8 * bins * channels.length);
  const view = new DataView(buffer);
  view.setUint32(0, 0x31535353, true);
  view.setUint32(4, sequence, true);
  view.setUint8(24, maskOf(channels));
  view.setUint8(25, linear ? 1 : 0);
  view.setUint8(26, window);
  view.setUint32(28, bins, true);
  view.setFloat64(32, binHz, true);
  view.setFloat32(40, 10, true);
  view.setFloat32(44, 10, true);
  view.setUint32(48, setupRevision, true);
  const levelsStart = 56 + 4 * channels.length;
  channels.forEach((channel, slot) => {
    view.setUint32(56 + 4 * slot, peakBins[slot] ?? NO_PEAK, true);
    for (let bin = 0; bin < bins; bin++) {
      view.setFloat32(levelsStart + 4 * (2 * slot * bins + bin), -channel, true);
      view.setFloat32(levelsStart + 4 * ((2 * slot + 1) * bins + bin), channel, true);
    }
  });
  return buffer;
}
