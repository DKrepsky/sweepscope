import { get } from 'svelte/store';
import { describe, expect, it, vi } from 'vitest';
import { createViewStore } from './view-store';

function setup() {
  const setSpectrumActive = vi.fn();
  return { store: createViewStore({ setSpectrumActive }), setSpectrumActive };
}

describe('view store', () => {
  it('given a new store, when read, then the view is Yt', () => {
    expect(get(setup().store)).toBe('yt');
  });

  it('given Yt, when FFT is entered, then the backend is told the spectrum is active', () => {
    const { store, setSpectrumActive } = setup();

    store.set('fft');

    expect(get(store)).toBe('fft');
    expect(setSpectrumActive).toHaveBeenCalledExactlyOnceWith(true);
  });

  it('given FFT, when Yt is entered, then the backend is told the spectrum is inactive', () => {
    const { store, setSpectrumActive } = setup();
    store.set('fft');

    store.set('yt');

    expect(setSpectrumActive).toHaveBeenLastCalledWith(false);
  });

  it('given Yt, when XY is entered and left, then the backend hears nothing', () => {
    const { store, setSpectrumActive } = setup();

    store.set('xy');
    store.set('yt');

    expect(setSpectrumActive).not.toHaveBeenCalled();
  });

  it('given FFT, when XY is entered, then the backend is told the spectrum is inactive', () => {
    const { store, setSpectrumActive } = setup();
    store.set('fft');

    store.set('xy');

    expect(setSpectrumActive).toHaveBeenLastCalledWith(false);
    expect(setSpectrumActive).toHaveBeenCalledTimes(2);
  });

  it('given FFT, when FFT is chosen again, then nothing is sent', () => {
    const { store, setSpectrumActive } = setup();
    store.set('fft');

    store.set('fft');

    expect(setSpectrumActive).toHaveBeenCalledOnce();
  });
});
