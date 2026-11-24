import type { SpectrumWindow } from '../../api';
import { channelsInMask, type TraceChannel } from './trace-channel';

const MAGIC = 0x31535353;
const HEADER_BYTES = 56;
const NO_PEAK = 0xffffffff;
const FLAG_LINEAR = 1;
const WINDOWS: SpectrumWindow[] = ['rectangular', 'hann', 'hamming', 'blackman'];

export interface DecodedSpectrum {
  sequence: number;
  mask: number;
  linear: boolean;
  window: SpectrumWindow;
  bins: number;
  binHz: number;
  /** Level at the top edge, dBV or volts rms. */
  top: number;
  unitsPerDiv: number;
  setupRevision: number;
  channels: {
    channel: TraceChannel;
    /** The backend's peak bin; undefined when the spectrum has none. */
    peakBin: number | undefined;
    /** dBV or volts rms, by `linear`; views of the received buffer. */
    levels: Float32Array;
    /** Divisions from the top edge, so the graph draws without computing. */
    divisions: Float32Array;
  }[];
}

/** Layout of ADR 0018 item 4 (little endian); every array offset is a multiple of 4. */
export function decodeSpectrum(buffer: ArrayBuffer): DecodedSpectrum {
  if (buffer.byteLength < HEADER_BYTES) throw new Error('Spectrum is shorter than its header');
  const view = new DataView(buffer);
  if (view.getUint32(0, true) !== MAGIC) throw new Error('Unknown spectrum magic');
  const mask = view.getUint8(24);
  const window = WINDOWS[view.getUint8(26)];
  if (!window) throw new Error('Unknown spectrum window');
  const bins = view.getUint32(28, true);
  const present = channelsInMask(mask);
  const peaksEnd = HEADER_BYTES + 4 * present.length;
  if (buffer.byteLength !== peaksEnd + 8 * bins * present.length) {
    throw new Error('Spectrum length does not match its bin count');
  }
  const arrayAt = (slot: number, part: 0 | 1) =>
    new Float32Array(buffer, peaksEnd + 4 * bins * (2 * slot + part), bins);
  return {
    sequence: view.getUint32(4, true),
    mask,
    linear: (view.getUint8(25) & FLAG_LINEAR) !== 0,
    window,
    bins,
    binHz: view.getFloat64(32, true),
    top: view.getFloat32(40, true),
    unitsPerDiv: view.getFloat32(44, true),
    setupRevision: view.getUint32(48, true),
    channels: present.map((channel, slot) => {
      const peak = view.getUint32(HEADER_BYTES + 4 * slot, true);
      return {
        channel,
        peakBin: peak === NO_PEAK ? undefined : peak,
        levels: arrayAt(slot, 0),
        divisions: arrayAt(slot, 1),
      };
    }),
  };
}
