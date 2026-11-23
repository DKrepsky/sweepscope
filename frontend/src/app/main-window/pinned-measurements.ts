import { readonly, writable, type Readable } from 'svelte/store';
import type { BackendClient } from '../../api';
import { loadUiState, saveUiState } from '../ui-state';

const NAMESPACE = 'measurements';
const VERSION = 1;

export const DEFAULT_PINNED: readonly string[] = ['frequency', 'period', 'vpp', 'vrms', 'riseTime'];

export interface PinnedMeasurements extends Readable<readonly string[]> {
  /** Restores the saved choice; a missing or unreadable blob keeps the defaults. */
  load(): Promise<void>;
  /** Pins or unpins one row and saves the choice; a failed save keeps the choice for this session. */
  set(id: string, pinned: boolean): Promise<void>;
}

function parse(json: string): string[] | undefined {
  try {
    const value: unknown = JSON.parse(json);
    const pinned = (value as { pinned?: unknown } | null)?.pinned;
    return Array.isArray(pinned) && pinned.every((id) => typeof id === 'string') ? (pinned as string[]) : undefined;
  } catch {
    return undefined;
  }
}

export function createPinnedMeasurements({ client }: { client: BackendClient }): PinnedMeasurements {
  const held = writable<readonly string[]>(DEFAULT_PINNED);
  let current: readonly string[] = DEFAULT_PINNED;
  const apply = (next: readonly string[]) => {
    current = next;
    held.set(next);
  };

  return {
    subscribe: readonly(held).subscribe,
    load: async () => {
      const entry = await loadUiState(client, NAMESPACE).catch(() => null);
      const saved = entry && entry.version === VERSION ? parse(entry.json) : undefined;
      if (saved) apply(saved);
    },
    set: async (id, pinned) => {
      apply(pinned ? [...current.filter((other) => other !== id), id] : current.filter((other) => other !== id));
      await saveUiState(client, {
        namespace: NAMESPACE,
        version: VERSION,
        json: JSON.stringify({ pinned: current }),
      }).catch(() => undefined);
    },
  };
}
