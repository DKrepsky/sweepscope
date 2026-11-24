import type { TraceChannel } from './trace-channel';

export interface TestFrame {
  sequence?: number;
  channels?: TraceChannel[];
  samplesPerChannel?: number;
  triggerIndex?: number;
  sentUs?: number;
  setupRevision?: number;
}

/** Builds an ADR 0011 frame; channel `c` holds the constant `c` so a test can tell channels apart. */
export function buildFrame({
  sequence = 1,
  channels = [1],
  samplesPerChannel = 4,
  triggerIndex = -1,
  sentUs = 0,
  setupRevision = 1,
}: TestFrame = {}): ArrayBuffer {
  const buffer = new ArrayBuffer(48 + 4 * samplesPerChannel * channels.length);
  const view = new DataView(buffer);
  view.setUint32(0, 0x31465353, true);
  view.setUint32(4, sequence, true);
  view.setFloat64(16, sentUs, true);
  view.setUint8(
    24,
    channels.reduce((mask, channel) => mask | (1 << (channel - 1)), 0),
  );
  view.setUint32(28, samplesPerChannel, true);
  view.setFloat64(32, 4e-9, true);
  view.setInt32(40, triggerIndex, true);
  view.setUint32(44, setupRevision, true);
  channels.forEach((channel, slot) => {
    for (let i = 0; i < samplesPerChannel; i++) {
      view.setFloat32(48 + 4 * (slot * samplesPerChannel + i), channel, true);
    }
  });
  return buffer;
}
