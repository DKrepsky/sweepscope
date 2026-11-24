import { get } from 'svelte/store';
import { describe, expect, it } from 'vitest';
import { createXyAxesStore } from './xy-axes-store';

describe('xy axes store', () => {
  it('given a new store, when read, then X is CH1 and Y is CH2', () => {
    expect(get(createXyAxesStore())).toEqual({ x: 1, y: 2 });
  });

  it('given X CH1 and Y CH2, when X is set to CH3, then X changes', () => {
    const store = createXyAxesStore();

    store.setX(3);

    expect(get(store)).toEqual({ x: 3, y: 2 });
  });

  it('given Y is CH2, when X is set to CH2, then it is refused', () => {
    const store = createXyAxesStore();

    store.setX(2);

    expect(get(store)).toEqual({ x: 1, y: 2 });
  });

  it('given X is CH1, when Y is set to CH1, then it is refused', () => {
    const store = createXyAxesStore();

    store.setY(1);

    expect(get(store)).toEqual({ x: 1, y: 2 });
  });

  it('given channels 3 and 4 enabled, when defaults are picked, then they become the axes', () => {
    const store = createXyAxesStore();

    store.pickDefaults([3, 4]);

    expect(get(store)).toEqual({ x: 3, y: 4 });
  });

  it('given both chosen channels still enabled, when defaults are picked, then the choice is kept', () => {
    const store = createXyAxesStore();
    store.setX(3);

    store.pickDefaults([1, 2, 3]);

    expect(get(store)).toEqual({ x: 3, y: 2 });
  });

  it('given fewer than two channels enabled, when defaults are picked, then the axes stay', () => {
    const store = createXyAxesStore();

    store.pickDefaults([4]);

    expect(get(store)).toEqual({ x: 1, y: 2 });
  });
});
