import { readonly, writable, type Readable } from 'svelte/store';
import type { ViewTab } from '../main-window/types';

export type View = ViewTab;

export interface ViewStore extends Readable<View> {
  set(view: View): void;
}

interface Deps {
  /** The backend computes spectra only while it is told the FFT view is shown. */
  setSpectrumActive: (active: boolean) => void;
}

/** Presentation state: only crossing the FFT boundary reaches the backend. */
export function createViewStore({ setSpectrumActive }: Deps): ViewStore {
  const held = writable<View>('yt');
  let current: View = 'yt';
  return {
    subscribe: readonly(held).subscribe,
    set: (next) => {
      if (next === current) return;
      const crossesFft = (current === 'fft') !== (next === 'fft');
      current = next;
      held.set(next);
      if (crossesFft) setSpectrumActive(next === 'fft');
    },
  };
}
