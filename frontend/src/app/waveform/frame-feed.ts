import { readonly, writable, type Readable } from 'svelte/store';
import type { BackendClient, StreamHandle } from '../../api';
import { decodeFrame, type DecodedFrame } from './decode-frame';

export interface FrameFeedStats {
  received: number;
  published: number;
  /** Frames dropped because a newer one arrived before the animation frame. */
  replaced: number;
  rejected: number;
  /** Clock minus the shell's send stamp at the last publish; 0 before the first. */
  latencyMs: number;
}

export interface FrameFeed {
  frame: Readable<DecodedFrame | undefined>;
  /** The last frame stays after `stop` so the graph can freeze on it. */
  start(): void;
  stop(): void;
  stats(): FrameFeedStats;
}

interface Deps {
  client: BackendClient;
  /** Calls back once, at the next animation frame. */
  requestFrame: (callback: () => void) => void;
  /** Milliseconds since the Unix epoch; the frame stamps use the same clock. */
  now: () => number;
}

/** The animation loop, not the transport, sets the redraw cadence: the newest frame is published at most once per animation frame. */
export function createFrameFeed({ client, requestFrame, now }: Deps): FrameFeed {
  const held = writable<DecodedFrame | undefined>(undefined);
  const counts = { received: 0, published: 0, replaced: 0, rejected: 0, latencyMs: 0 };
  let stream: StreamHandle | undefined;
  let newest: DecodedFrame | undefined;

  const publish = () => {
    const frame = newest;
    newest = undefined;
    if (!frame || !stream) return;
    counts.published++;
    counts.latencyMs = now() - frame.sentUs / 1000;
    held.set(frame);
  };

  const accept = (data: ArrayBuffer) => {
    let frame: DecodedFrame;
    try {
      frame = decodeFrame(data);
    } catch {
      counts.rejected++;
      return;
    }
    counts.received++;
    if (newest) counts.replaced++;
    else requestFrame(publish);
    newest = frame;
  };

  return {
    frame: readonly(held),
    start: () => {
      stream ??= client.openStream('frames', accept);
    },
    stop: () => {
      stream?.close();
      stream = undefined;
      newest = undefined;
    },
    stats: () => ({ ...counts }),
  };
}
