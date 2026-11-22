import { mount } from 'svelte';
import { createTauriClient } from './api';
import { createTextScaleStore } from './app/text-scale';
import MainWindow from './pages/main-window/MainWindow.svelte';

const applyRootFontPercent = (percent: number) => {
  document.documentElement.style.fontSize = `${percent}%`;
};

async function start(): Promise<void> {
  const client = createTauriClient();
  const textScale = createTextScaleStore({ client, applyRootFontPercent });
  await textScale.load().catch(() => applyRootFontPercent(100));
  const target = document.getElementById('app');
  if (!target) throw new Error('Missing #app mount point');
  mount(MainWindow, { target });
}

void start();
