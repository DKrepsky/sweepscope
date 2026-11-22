import { writable, type Readable } from 'svelte/store';
import type { BackendClient, UiStateEntry } from '../../api';
import { loadUiState, saveUiState } from '../ui-state';

export type TextScalePercent = 100 | 125 | 150 | 200;
export const textScaleOptions: readonly TextScalePercent[] = [100, 125, 150, 200];

const NAMESPACE = 'app.text-scale';
const VERSION = 1;
const DEFAULT_PERCENT: TextScalePercent = 100;

export interface TextScaleStore extends Pick<Readable<TextScalePercent>, 'subscribe'> {
  load(): Promise<void>;
  /** Persists first, then updates the value; use `setTextScale`. */
  persist(percent: TextScalePercent): Promise<void>;
}

export interface TextScaleDeps {
  client: BackendClient;
  applyRootFontPercent: (percent: number) => void;
}

function parsePercent(entry: UiStateEntry | null): TextScalePercent {
  if (!entry || entry.version !== VERSION) return DEFAULT_PERCENT;
  try {
    const { percent } = JSON.parse(entry.json) as { percent?: unknown };
    return textScaleOptions.find((option) => option === percent) ?? DEFAULT_PERCENT;
  } catch {
    return DEFAULT_PERCENT;
  }
}

export function createTextScaleStore({ client, applyRootFontPercent }: TextScaleDeps): TextScaleStore {
  const value = writable<TextScalePercent>(DEFAULT_PERCENT);
  const adopt = (percent: TextScalePercent) => {
    value.set(percent);
    applyRootFontPercent(percent);
  };

  return {
    subscribe: value.subscribe,
    async load() {
      adopt(parsePercent(await loadUiState(client, NAMESPACE)));
    },
    async persist(percent) {
      await saveUiState(client, { namespace: NAMESPACE, version: VERSION, json: JSON.stringify({ percent }) });
      adopt(percent);
    },
  };
}

export function setTextScale(store: TextScaleStore, percent: TextScalePercent): Promise<void> {
  return store.persist(percent);
}
