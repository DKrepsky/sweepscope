import { get, writable } from 'svelte/store';
import { describe, expect, it } from 'vitest';
import { createLayoutTierStore, layoutTierFor, type ViewportSize, type ViewportSource } from './layout-tier';
import { createResizeObserverViewport } from './viewport-source';

const rootFont = (scale: number) => (16 * scale) / 100;
const widthTier = (width: number, scale: number) => layoutTierFor(width, 900, rootFont(scale)).width;

describe('layoutTierFor', () => {
  const rows: [number, number[]][] = [
    [3840, [100, 125, 150, 200]],
    [2560, [100, 125, 150, 200]],
    [2048, [100, 125, 150, 200]],
    [1920, [100, 125, 150, 200]],
    [1536, [100, 125, 150, 200]],
    [1280, [100, 125, 150, 200]],
    [960, [100, 125, 150, 200]],
  ];
  const expected: Record<number, string[]> = {
    3840: ['w1', 'w1', 'w1', 'w1'],
    2560: ['w1', 'w1', 'w1', 'w2'],
    2048: ['w1', 'w1', 'w2', 'w3'],
    1920: ['w1', 'w1', 'w2', 'scroll'],
    1536: ['w1', 'w3', 'w3', 'scroll'],
    1280: ['w2', 'w3', 'scroll', 'scroll'],
    960: ['scroll', 'scroll', 'scroll', 'scroll'],
  };

  for (const [width, scales] of rows) {
    scales.forEach((scale, index) => {
      it(`given a ${width} px window at ${scale} % text, when the tier is computed, then it is ${expected[width]![index]}`, () => {
        expect(widthTier(width, scale)).toBe(expected[width]![index]);
      });
    });
  }

  it.each([
    [90, 'w1'],
    [89.99, 'w2'],
    [80, 'w2'],
    [79.99, 'w3'],
    [64, 'w3'],
    [63.99, 'scroll'],
  ])('given a width of %s rem, when the tier is computed, then it is %s', (rem, tier) => {
    expect(layoutTierFor(rem * 16, 900, 16).width).toBe(tier);
  });

  it.each([
    [56, 'h1'],
    [55.99, 'h2'],
    [45, 'h2'],
    [44.99, 'h3'],
  ])('given a height of %s rem, when the tier is computed, then it is %s', (rem, tier) => {
    expect(layoutTierFor(1920, rem * 16, 16).height).toBe(tier);
  });
});

function fakeViewport(initial: ViewportSize) {
  let listener: ((size: ViewportSize) => void) | null = null;
  const source: ViewportSource = {
    subscribe(next) {
      listener = next;
      next(initial);
      return () => (listener = null);
    },
  };
  return { source, emit: (size: ViewportSize) => listener?.(size) };
}

describe('createLayoutTierStore', () => {
  it('given the viewport emits a new size, when the store is read, then it reflects the tier', () => {
    const viewport = fakeViewport({ width: 1920, height: 1080 });
    const store = createLayoutTierStore({ viewport: viewport.source, textScale: writable(100) });
    const seen: string[] = [];
    const stop = store.subscribe((tier) => seen.push(tier.width));

    viewport.emit({ width: 1280, height: 720 });
    stop();

    expect(seen).toEqual(['w1', 'w2']);
  });

  it('given the text scale changes from 100 to 200 at the same pixel size, when the store is read, then the tier is re-evaluated', () => {
    const textScale = writable(100);
    const viewport = fakeViewport({ width: 1920, height: 1080 });
    const store = createLayoutTierStore({ viewport: viewport.source, textScale });
    const stop = store.subscribe(() => {});

    textScale.set(200);

    expect(get(store)).toEqual({ width: 'scroll', height: 'h3' });
    stop();
  });
});

describe('createResizeObserverViewport', () => {
  it('given an element, when subscribed and resized, then the listener gets the current size and stops after unsubscribe', () => {
    const element = document.createElement('div');
    let size = { w: 800, h: 600 };
    Object.defineProperty(element, 'clientWidth', { get: () => size.w });
    Object.defineProperty(element, 'clientHeight', { get: () => size.h });
    let notify = () => {};
    let disconnected = false;
    const viewport = createResizeObserverViewport(element, (callback) => {
      notify = callback;
      return { observe: () => {}, disconnect: () => (disconnected = true) };
    });
    const seen: ViewportSize[] = [];

    const stop = viewport.subscribe((value) => seen.push(value));
    size = { w: 1000, h: 700 };
    notify();
    stop();

    expect(seen).toEqual([
      { width: 800, height: 600 },
      { width: 1000, height: 700 },
    ]);
    expect(disconnected).toBe(true);
  });
});
