import { writable, type Readable } from 'svelte/store';

export type CursorId = 'x1' | 'x2' | 'y1' | 'y2';

/** Fractions of the screen: x from the left, y from the top. A cursor that is absent is not shown. */
export interface CursorsState {
  x1?: number;
  x2?: number;
  y1?: number;
  y2?: number;
}

export interface CursorsStore extends Readable<CursorsState> {
  /** Adds all four at their defaults when none is present, otherwise clears. */
  toggle(): void;
  /** Moves a cursor that is present; a late move after a removal must not bring it back. */
  move(id: CursorId, position: number): void;
  remove(id: CursorId): void;
  clear(): void;
}

const DEFAULTS: Required<CursorsState> = { x1: 0.3, x2: 0.7, y1: 0.3, y2: 0.7 };

const clamp = (position: number) => Math.min(1, Math.max(0, position));

const isEmpty = (state: CursorsState) => Object.keys(state).length === 0;

export const createCursorsStore = (): CursorsStore => {
  const state = writable<CursorsState>({});
  return {
    subscribe: state.subscribe,
    toggle: () => state.update((current) => (isEmpty(current) ? { ...DEFAULTS } : {})),
    move: (id, position) => {
      if (!Number.isFinite(position)) return;
      state.update((current) => (current[id] === undefined ? current : { ...current, [id]: clamp(position) }));
    },
    remove: (id) =>
      state.update((current) => Object.fromEntries(Object.entries(current).filter(([key]) => key !== id))),
    clear: () => state.set({}),
  };
};
