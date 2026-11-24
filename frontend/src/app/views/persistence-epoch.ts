import { readonly, writable, type Readable } from 'svelte/store';

export interface PersistenceEpoch extends Readable<number> {
  clearPersistence(): void;
}

/** The graph drops its ghosts whenever the epoch changes. */
export function createPersistenceEpoch(): PersistenceEpoch {
  const held = writable(0);
  return { subscribe: readonly(held).subscribe, clearPersistence: () => held.update((epoch) => epoch + 1) };
}
