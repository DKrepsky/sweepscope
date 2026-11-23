import type { BackendClient } from '../../api';
import { createCursorsStore } from '../cursors';
import { createFileActions } from '../files';
import { createInstrumentCommands, createInstrumentStores } from '../instrument';
import { createFrameFeed, type FrameBudgetProbe } from '../waveform';
import { createLiveMainWindow, type LiveMainWindow } from './live-main-window';
import { createPinnedMeasurements } from './pinned-measurements';

interface Deps {
  client: BackendClient;
  notify: (kind: 'info' | 'warning' | 'error', text: string) => void;
  /** PNG bytes of the window as it looks now. */
  composeScreenshot: () => Promise<Uint8Array>;
  copyText: (text: string) => void;
  /** Milliseconds since the Unix epoch, the clock the frame stamps use. */
  now: () => number;
  requestAnimationFrame: (callback: () => void) => void;
  /** Resolves once the UI has applied what the last store change caused (Svelte's `tick`). */
  afterRender: () => Promise<void>;
  probe: FrameBudgetProbe;
}

export interface LiveApp {
  model: LiveMainWindow;
  start(): Promise<void>;
  stop(): void;
}

/** The canvas draws in the effect that follows the publish, so the probe's draw time ends after the render. */
const probedFrames =
  ({ requestAnimationFrame, afterRender, probe }: Deps) =>
  (publish: () => void) =>
    requestAnimationFrame(() => {
      probe.beginFrame();
      publish();
      void afterRender().then(() => probe.endFrame());
    });

export function createLiveApp(deps: Deps): LiveApp {
  const { client, notify, copyText, now, composeScreenshot } = deps;
  const stores = createInstrumentStores({ client });
  const commands = createInstrumentCommands({ client, notify });
  const feed = createFrameFeed({ client, requestFrame: probedFrames(deps), now });
  const cursors = createCursorsStore();
  const pinned = createPinnedMeasurements({ client });
  const files = createFileActions({ client, notify, composeScreenshot });
  const model = createLiveMainWindow({
    stores,
    commands,
    cursors,
    pinned,
    files,
    frames: feed.frame,
    copyText,
    notify,
    renderer: 'Canvas 2D',
  });

  return {
    model,
    start: async () => {
      feed.start();
      void pinned.load();
      await stores.start();
    },
    stop: () => {
      feed.stop();
      stores.stop();
    },
  };
}
