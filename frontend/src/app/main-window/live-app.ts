import type { BackendClient } from '../../api';
import { followAutosetOutcomes } from '../autoset';
import { createCursorsStore } from '../cursors';
import { createFileActions } from '../files';
import { createInstrumentCommands, createInstrumentStores, type InstrumentStores } from '../instrument';
import type { NotificationAction } from '../notifications';
import { inertAnalysisSources, type AnalyseCommands, type AnalysisSources } from '../views';
import { createFrameFeed, type FrameBudgetProbe } from '../waveform';
import { createLiveMainWindow, type LiveMainWindow } from './live-main-window';
import { createPinnedMeasurements } from './pinned-measurements';

interface Deps {
  client: BackendClient;
  notify: (kind: 'info' | 'warning' | 'error', text: string, action?: NotificationAction) => void;
  /** PNG bytes of the window as it looks now. */
  composeScreenshot: () => Promise<Uint8Array>;
  copyText: (text: string) => void;
  /** Milliseconds since the Unix epoch, the clock the frame stamps use. */
  now: () => number;
  requestAnimationFrame: (callback: () => void) => void;
  /** Resolves once the UI has applied what the last store change caused (Svelte's `tick`). */
  afterRender: () => Promise<void>;
  probe: FrameBudgetProbe;
  /** Feeds and commands of the analysis features; without them every analysis control stays unavailable. */
  analysis?: { sources: AnalysisSources; commands: AnalyseCommands };
}

const inertAnalyseCommands: AnalyseCommands = {
  armSingle: () => {},
  startAutoset: () => {},
  setMath: () => {},
  setPersistence: () => {},
  setSpectrum: () => {},
  startRecording: () => {},
  stopRecording: () => {},
};

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

/** Shows each setup notice once: the id is the notice's identity, so repeats and reconnects stay silent. */
function followSetupNotices({ instrument }: InstrumentStores, notify: Deps['notify']) {
  let lastShownId: number | undefined;
  return () =>
    instrument.subscribe((state) => {
      const notice = state?.setupNotice;
      if (!notice || notice.id === lastShownId) return;
      lastShownId = notice.id;
      notify('warning', notice.text);
    });
}

/** Every toast the backend state causes: setup notices and auto-setup outcomes. Returns the function that stops them. */
function followBackendMessages(stores: InstrumentStores, notify: Deps['notify'], retryAutoset: () => void) {
  const followNotices = followSetupNotices(stores, notify);
  return () => {
    const stops = [followNotices(), followAutosetOutcomes({ status: stores.status, notify, retry: retryAutoset })];
    return () => stops.forEach((stop) => stop());
  };
}

export function createLiveApp(deps: Deps): LiveApp {
  const { client, notify, copyText, now, composeScreenshot } = deps;
  const stores = createInstrumentStores({ client });
  const commands = createInstrumentCommands({ client, notify });
  const feed = createFrameFeed({ client, requestFrame: probedFrames(deps), now });
  const cursors = createCursorsStore();
  const pinned = createPinnedMeasurements({ client });
  const files = createFileActions({ client, notify, composeScreenshot });
  const followMessages = followBackendMessages(stores, notify, () => model.startAutoset());
  let stopMessages = () => {};
  const model = createLiveMainWindow({
    stores,
    analysis: deps.analysis?.sources ?? inertAnalysisSources(),
    commands,
    analyseCommands: deps.analysis?.commands ?? inertAnalyseCommands,
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
      stopMessages();
      stopMessages = followMessages();
      void pinned.load();
      await stores.start();
    },
    stop: () => {
      feed.stop();
      stopMessages();
      stores.stop();
    },
  };
}
