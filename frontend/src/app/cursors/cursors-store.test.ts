import { get } from 'svelte/store';
import { describe, expect, it } from 'vitest';
import { createCursorsStore } from './cursors-store';

const FOUR = { x1: 0.3, x2: 0.7, y1: 0.3, y2: 0.7 };

describe('cursors store', () => {
  it('given no cursors, when toggled, then four cursors appear at their defaults', () => {
    const store = createCursorsStore();
    store.toggle();
    expect(get(store)).toEqual(FOUR);
  });

  it('given four cursors, when toggled, then none remain', () => {
    const store = createCursorsStore();
    store.toggle();
    store.toggle();
    expect(get(store)).toEqual({});
  });

  it('given one cursor left, when toggled, then all are cleared', () => {
    const store = createCursorsStore();
    store.toggle();
    store.remove('x1');
    store.remove('x2');
    store.remove('y1');
    store.toggle();
    expect(get(store)).toEqual({});
  });

  it('given a cursor, when moved beyond the screen, then it clamps to the edge', () => {
    const store = createCursorsStore();
    store.toggle();
    store.move('x1', 1.4);
    store.move('y2', -0.2);
    expect(get(store)).toMatchObject({ x1: 1, y2: 0 });
  });

  it('given a cursor, when moved inside the screen, then only that cursor changes', () => {
    const store = createCursorsStore();
    store.toggle();
    store.move('x2', 0.55);
    expect(get(store)).toEqual({ ...FOUR, x2: 0.55 });
  });

  it('given a position that is not a number, when moved, then the cursor stays', () => {
    const store = createCursorsStore();
    store.toggle();
    store.move('x1', Number.NaN);
    expect(get(store)).toEqual(FOUR);
  });

  it('given four cursors, when one is removed, then only that one disappears', () => {
    const store = createCursorsStore();
    store.toggle();
    store.remove('y1');
    expect(get(store)).toEqual({ x1: 0.3, x2: 0.7, y2: 0.7 });
  });

  it('given a removed cursor, when a late move arrives, then it does not come back', () => {
    const store = createCursorsStore();
    store.toggle();
    store.remove('x1');
    store.move('x1', 0.5);
    expect(get(store).x1).toBeUndefined();
  });

  it('given cursors, when cleared, then none remain', () => {
    const store = createCursorsStore();
    store.toggle();
    store.clear();
    expect(get(store)).toEqual({});
  });
});
