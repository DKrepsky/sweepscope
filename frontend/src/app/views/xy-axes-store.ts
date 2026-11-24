import { readonly, writable, type Readable } from 'svelte/store';
import type { ChannelNumber } from '../main-window/types';

export interface XyAxes {
  x: ChannelNumber;
  y: ChannelNumber;
}

export interface XyAxesStore extends Readable<XyAxes> {
  setX(channel: ChannelNumber): void;
  setY(channel: ChannelNumber): void;
  /** Entering XY: the first two enabled channels, kept when both chosen channels are still enabled. */
  pickDefaults(enabled: readonly ChannelNumber[]): void;
}

const INITIAL: XyAxes = { x: 1, y: 2 };

/** The other axis' channel is refused, so the two axes never share a channel. */
export function createXyAxesStore(): XyAxesStore {
  const held = writable<XyAxes>(INITIAL);
  let current = INITIAL;
  const apply = (next: XyAxes) => {
    current = next;
    held.set(next);
  };
  return {
    subscribe: readonly(held).subscribe,
    setX: (x) => {
      if (x !== current.y) apply({ ...current, x });
    },
    setY: (y) => {
      if (y !== current.x) apply({ ...current, y });
    },
    pickDefaults: (enabled) => {
      if (enabled.includes(current.x) && enabled.includes(current.y)) return;
      const [x, y] = enabled;
      if (x !== undefined && y !== undefined) apply({ x, y });
    },
  };
}
