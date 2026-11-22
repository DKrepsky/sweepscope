import { describe, expect, it, vi } from 'vitest';
import { watchDevicePixelRatio } from './device-pixel-ratio';
import type { RatioWindow } from './device-pixel-ratio';

function fakeWindow(initial: number) {
  const listeners = new Map<string, Set<() => void>>();
  const win: RatioWindow = {
    devicePixelRatio: initial,
    matchMedia: (query) => {
      const set = listeners.get(query) ?? new Set();
      listeners.set(query, set);
      return {
        addEventListener: (_: string, l: EventListenerOrEventListenerObject) => set.add(l as () => void),
        removeEventListener: (_: string, l: EventListenerOrEventListenerObject) => void set.delete(l as () => void),
      };
    },
  };
  const change = (to: number) => {
    const query = `(resolution: ${win.devicePixelRatio}dppx)`;
    win.devicePixelRatio = to;
    [...(listeners.get(query) ?? [])].forEach((l) => l());
  };
  return { win, change };
}

describe('watchDevicePixelRatio', () => {
  it('given a watcher, when the ratio changes twice, then the callback gets each new ratio', () => {
    const { win, change } = fakeWindow(1);
    const onChange = vi.fn();
    watchDevicePixelRatio(win, onChange);
    change(1.5);
    change(2);
    expect(onChange.mock.calls).toEqual([[1.5], [2]]);
  });

  it('given a stopped watcher, when the ratio changes, then the callback is not called', () => {
    const { win, change } = fakeWindow(1);
    const onChange = vi.fn();
    watchDevicePixelRatio(win, onChange)();
    change(2);
    expect(onChange).not.toHaveBeenCalled();
  });
});
