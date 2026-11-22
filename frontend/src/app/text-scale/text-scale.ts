import { writable, type Readable } from 'svelte/store';
import type { BackendClient, UiStateEntry } from '../../api';
import type { Notifications } from '../notifications';
import { loadUiState, saveUiState } from '../ui-state';

export type TextScalePercent = 100 | 125 | 150 | 200;
export const textScaleOptions: readonly TextScalePercent[] = [100, 125, 150, 200];

const NAMESPACE = 'app.text-scale';
const VERSION = 1;
const DEFAULT_PERCENT: TextScalePercent = 100;

export type SaveResult = 'saved' | 'not-saved';

export interface TextScaleStore extends Pick<Readable<TextScalePercent>, 'subscribe'> {
  load(): Promise<void>;
  /** Applies the scale for the session, then saves it; never rejects. Use `setTextScale`. */
  persist(percent: TextScalePercent): Promise<SaveResult>;
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
      adopt(percent);
      try {
        await saveUiState(client, { namespace: NAMESPACE, version: VERSION, json: JSON.stringify({ percent }) });
        return 'saved';
      } catch {
        return 'not-saved';
      }
    },
  };
}

export function setTextScale(store: TextScaleStore, percent: TextScalePercent): Promise<SaveResult> {
  return store.persist(percent);
}

export const NOT_SAVED_TEXT = 'Text size changed but could not be saved. It resets when the app restarts.';

/** A failed save keeps the new scale for the session and tells the user (spec 11.4). */
export async function chooseTextScale(
  store: TextScaleStore,
  notifications: Notifications,
  percent: TextScalePercent,
): Promise<void> {
  if ((await setTextScale(store, percent)) === 'not-saved') {
    notifications.notify({ kind: 'warning', text: NOT_SAVED_TEXT });
  }
}
