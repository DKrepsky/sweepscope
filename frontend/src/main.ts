import './design-system/tokens.css';
import './design-system/base.css';
import { mount } from 'svelte';
import { createTauriClient } from './api';
import { createLayoutTierStore, createResizeObserverViewport } from './app/layout';
import { createMockMainWindow } from './app/main-window';
import { createTextScaleStore, setTextScale } from './app/text-scale';
import MainWindow from './pages/main-window/MainWindow.svelte';

const applyRootFontPercent = (percent: number) => {
  document.documentElement.style.fontSize = `${percent}%`;
};

async function start(): Promise<void> {
  const client = createTauriClient();
  const textScale = createTextScaleStore({ client, applyRootFontPercent });
  await textScale.load().catch(() => applyRootFontPercent(100));
  const tier = createLayoutTierStore({
    viewport: createResizeObserverViewport(document.documentElement),
    textScale,
  });
  const model = createMockMainWindow();
  const target = document.getElementById('app');
  if (!target) throw new Error('Missing #app mount point');
  mount(MainWindow, {
    target,
    props: {
      model,
      tier,
      textScale,
      ontogglerun: model.toggleRun,
      ontextscale: (percent) => void setTextScale(textScale, percent),
    },
  });
}

void start();
