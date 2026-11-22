import type { ChannelNumber } from '../main-window/types';

const MAGIC = 0x31465353;
const HEADER_BYTES = 48;
const FLAG_TRIGGERED = 1;
const FLAG_AUTO = 2;
const CHANNELS: ChannelNumber[] = [1, 2, 3, 4];

export interface DecodedFrame {
  sequence: number;
  producedUs: number;
  sentUs: number;
  triggered: boolean;
  auto: boolean;
  samplesPerChannel: number;
  sampleIntervalS: number;
  /** Inside the sent window; undefined when the hardware gave none or it is off the window. */
  triggerIndex: number | undefined;
  setupRevision: number;
  /** Divisions from the top edge, views of the received buffer. */
  channels: { channel: ChannelNumber; samples: Float32Array }[];
}

/** Layout of ADR 0011 (little endian). The sample arrays are 4-byte aligned by the 48-byte header. */
export function decodeFrame(buffer: ArrayBuffer): DecodedFrame {
  if (buffer.byteLength < HEADER_BYTES) throw new Error('Frame is shorter than its header');
  const view = new DataView(buffer);
  if (view.getUint32(0, true) !== MAGIC) throw new Error('Unknown frame magic');
  const mask = view.getUint8(24);
  const flags = view.getUint8(25);
  const samplesPerChannel = view.getUint32(28, true);
  const present = CHANNELS.filter((_, bit) => (mask & (1 << bit)) !== 0);
  if (buffer.byteLength !== HEADER_BYTES + 4 * samplesPerChannel * present.length) {
    throw new Error('Frame length does not match its sample count');
  }
  const triggerIndex = view.getInt32(40, true);
  return {
    sequence: view.getUint32(4, true),
    producedUs: view.getFloat64(8, true),
    sentUs: view.getFloat64(16, true),
    triggered: (flags & FLAG_TRIGGERED) !== 0,
    auto: (flags & FLAG_AUTO) !== 0,
    samplesPerChannel,
    sampleIntervalS: view.getFloat64(32, true),
    triggerIndex: triggerIndex < 0 ? undefined : triggerIndex,
    setupRevision: view.getUint32(44, true),
    channels: present.map((channel, slot) => ({
      channel,
      samples: new Float32Array(buffer, HEADER_BYTES + 4 * slot * samplesPerChannel, samplesPerChannel),
    })),
  };
}
