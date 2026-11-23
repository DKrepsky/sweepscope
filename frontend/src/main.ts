import './design-system/tokens.css';
import './design-system/base.css';
import { mount, tick } from 'svelte';
import { createTauriClient } from './api';
import { createCopyText } from './app/clipboard';
import { createLayoutTierStore, createResizeObserverViewport } from './app/layout';
import { createLiveApp } from './app/main-window';
import { createNotifications } from './app/notifications';
import { chooseTextScale, createTextScaleStore } from './app/text-scale';
import { consoleReport, createFrameBudgetProbe, type FrameBudgetProbe } from './app/waveform';
import MainWindow from './pages/main-window/MainWindow.svelte';

const applyRootFontPercent = (percent: number) => {
  document.documentElement.style.fontSize = `${percent}%`;
};

const idleProbe: FrameBudgetProbe = { beginFrame: () => {}, endFrame: () => {} };

/** The budget summary is a development aid; release builds do not measure. */
const frameProbe = (): FrameBudgetProbe =>
  import.meta.env.DEV ? createFrameBudgetProbe({ now: () => performance.now(), report: consoleReport }) : idleProbe;

async function start(): Promise<void> {
  const client = createTauriClient();
  const textScale = createTextScaleStore({ client, applyRootFontPercent });
  await textScale.load().catch(() => applyRootFontPercent(100));
  const tier = createLayoutTierStore({
    viewport: createResizeObserverViewport(document.documentElement),
    textScale,
  });
  const notifications = createNotifications();
  const notify = (kind: 'error', text: string) => notifications.notify({ kind, text });
  const app = createLiveApp({
    client,
    notify,
    copyText: createCopyText({
      write: (text) => navigator.clipboard.writeText(text),
      notify: notifications.notify,
    }),
    now: () => Date.now(),
    requestAnimationFrame: (callback) => void window.requestAnimationFrame(callback),
    afterRender: tick,
    probe: frameProbe(),
  });
  const target = document.getElementById('app');
  if (!target) throw new Error('Missing #app mount point');
  mount(MainWindow, {
    target,
    props: {
      model: app.model,
      actions: app.model,
      tier,
      textScale,
      notifications,
      onnotificationdismiss: notifications.dismiss,
      ontextscale: (percent) => void chooseTextScale(textScale, notifications, percent),
    },
  });
  await app.start().catch(() => notify('error', 'Could not reach the instrument service'));
}

void start();
