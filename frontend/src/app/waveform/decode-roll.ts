import { channelsInMask, type TraceChannel } from './trace-channel';

const MAGIC = 0x314c5353;
const HEADER_BYTES = 32;
const KINDS = ['samples', 'gap', 'closed'] as const;
const CLOSE_REASONS = ['stopped', 'leftRoll', 'disconnected', 'setupChanged'] as const;

export type RollItemKind = (typeof KINDS)[number];
export type RollCloseReason = (typeof CLOSE_REASONS)[number];

export interface RollItem {
  sequence: number;
  kind: RollItemKind;
  session: number;
  mask: number;
  /** Index of the first sample; for a gap, the index just before the missed samples (`after`). */
  firstIndex: number;
  count: number;
  /** Samples a gap stands for; 0 for the other kinds. */
  missed: number;
  closeReason: RollCloseReason | undefined;
  /** Divisions from the top edge, views of the received buffer; empty unless the kind is `samples`. */
  channels: { channel: TraceChannel; samples: Float32Array }[];
}

/** Layout of ADR 0017 item 5 (little endian). */
export function decodeRollItem(buffer: ArrayBuffer): RollItem {
  if (buffer.byteLength < HEADER_BYTES) throw new Error('Roll item is shorter than its header');
  const view = new DataView(buffer);
  if (view.getUint32(0, true) !== MAGIC) throw new Error('Unknown roll item magic');
  const kind = KINDS[view.getUint8(8)];
  if (!kind) throw new Error('Unknown roll item kind');
  const mask = view.getUint8(9);
  const count = view.getUint32(12, true);
  const present = kind === 'samples' ? channelsInMask(mask) : [];
  if (buffer.byteLength !== HEADER_BYTES + 4 * count * present.length) {
    throw new Error('Roll item length does not match its sample count');
  }
  return {
    sequence: view.getUint32(4, true),
    kind,
    session: view.getUint32(28, true),
    mask,
    firstIndex: Number(view.getBigUint64(16, true)),
    count,
    missed: view.getUint32(24, true),
    closeReason: kind === 'closed' ? CLOSE_REASONS[view.getUint8(10)] : undefined,
    channels: present.map((channel, slot) => ({
      channel,
      samples: new Float32Array(buffer, HEADER_BYTES + 4 * slot * count, count),
    })),
  };
}
