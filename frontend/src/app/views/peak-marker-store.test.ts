import { get } from 'svelte/store';
import { describe, expect, it } from 'vitest';
import { createPeakMarkerStore } from './peak-marker-store';

describe('peak marker store', () => {
  it('given a new store, when read, then the marker is automatic on CH1', () => {
    expect(get(createPeakMarkerStore())).toMatchObject({ mode: 'auto', channel: 1 });
  });

  it('given automatic mode, when the user places the marker, then the mode becomes manual at that bin', () => {
    const store = createPeakMarkerStore();

    store.place(40, 100);

    expect(get(store)).toMatchObject({ mode: 'manual', bin: 40 });
  });

  it('given automatic mode, when the marker is moved by bins, then the mode becomes manual', () => {
    const store = createPeakMarkerStore();

    store.moveBy(1, 100);

    expect(get(store)).toMatchObject({ mode: 'manual', bin: 1 });
  });

  it('given bin 5, when moved by minus 10, then it stops at bin 0', () => {
    const store = createPeakMarkerStore();
    store.place(5, 100);

    store.moveBy(-10, 100);

    expect(get(store).bin).toBe(0);
  });

  it('given 100 bins, when placed beyond the end, then it stops at the last bin', () => {
    const store = createPeakMarkerStore();

    store.place(500, 100);

    expect(get(store).bin).toBe(99);
  });

  it('given no spectrum bins, when placed, then the bin is 0', () => {
    const store = createPeakMarkerStore();

    store.place(7, 0);

    expect(get(store).bin).toBe(0);
  });

  it('given a mode choice, when set to off, then the marker is off and keeps its channel', () => {
    const store = createPeakMarkerStore();
    store.follow(3);

    store.setMode('off');

    expect(get(store)).toMatchObject({ mode: 'off', channel: 3 });
  });

  it('given a manual marker, when the followed channel changes, then the mode stays manual', () => {
    const store = createPeakMarkerStore();
    store.place(10, 100);

    store.follow(2);

    expect(get(store)).toMatchObject({ mode: 'manual', channel: 2, bin: 10 });
  });
});
