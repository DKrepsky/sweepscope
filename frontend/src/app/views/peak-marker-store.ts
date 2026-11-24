import { readonly, writable, type Readable } from 'svelte/store';
import type { ChannelNumber, PeakMode } from '../main-window/types';

export interface PeakMarker {
  mode: PeakMode;
  channel: ChannelNumber;
  /** The manual bin; in auto mode the marker shows the backend's peak bin instead. */
  bin: number;
}

export interface PeakMarkerStore extends Readable<PeakMarker> {
  setMode(mode: PeakMode): void;
  follow(channel: ChannelNumber): void;
  moveBy(bins: number, binCount: number): void;
  place(bin: number, binCount: number): void;
}

const INITIAL: PeakMarker = { mode: 'auto', channel: 1, bin: 0 };

const clamp = (bin: number, binCount: number) => Math.min(Math.max(0, binCount - 1), Math.max(0, Math.round(bin)));

/** A user placement or move always makes the marker manual; the bin is a spectrum index, never a frequency. */
export function createPeakMarkerStore(): PeakMarkerStore {
  const held = writable<PeakMarker>(INITIAL);
  let current = INITIAL;
  const apply = (next: PeakMarker) => {
    current = next;
    held.set(next);
  };
  return {
    subscribe: readonly(held).subscribe,
    setMode: (mode) => apply({ ...current, mode }),
    follow: (channel) => apply({ ...current, channel }),
    moveBy: (bins, binCount) => apply({ ...current, mode: 'manual', bin: clamp(current.bin + bins, binCount) }),
    place: (bin, binCount) => apply({ ...current, mode: 'manual', bin: clamp(bin, binCount) }),
  };
}
