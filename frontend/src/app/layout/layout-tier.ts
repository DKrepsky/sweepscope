import { readable, type Readable } from 'svelte/store';

export type WidthTier = 'w1' | 'w2' | 'w3' | 'scroll';
export type HeightTier = 'h1' | 'h2' | 'h3';

export interface LayoutTier {
  width: WidthTier;
  height: HeightTier;
}

export interface ViewportSize {
  width: number;
  height: number;
}

/** Emits the current size synchronously on subscribe, then on every change. */
export interface ViewportSource {
  subscribe(listener: (size: ViewportSize) => void): () => void;
}

interface ScaleSource {
  subscribe(listener: (percent: number) => void): () => void;
}

const BASE_FONT_PX = 16;

/** Thresholds of design 3.2, in rem. This is the only place they live. */
const WIDTH_TIERS: readonly [number, WidthTier][] = [
  [90, 'w1'],
  [80, 'w2'],
  [64, 'w3'],
];
const HEIGHT_TIERS: readonly [number, HeightTier][] = [
  [56, 'h1'],
  [45, 'h2'],
];

function pick<T>(rem: number, tiers: readonly [number, T][], fallback: T): T {
  return tiers.find(([minimum]) => rem >= minimum)?.[1] ?? fallback;
}

export function layoutTierFor(widthPx: number, heightPx: number, rootFontPx: number): LayoutTier {
  return {
    width: pick(widthPx / rootFontPx, WIDTH_TIERS, 'scroll'),
    height: pick(heightPx / rootFontPx, HEIGHT_TIERS, 'h3'),
  };
}

export const INITIAL_TIER: LayoutTier = { width: 'w1', height: 'h1' };

const sameTier = (a: LayoutTier, b: LayoutTier) => a.width === b.width && a.height === b.height;

export interface LayoutTierDeps {
  viewport: ViewportSource;
  /** Text scale in percent; the tier is re-evaluated when it changes (ADR 0005). */
  textScale: ScaleSource;
}

export function createLayoutTierStore({ viewport, textScale }: LayoutTierDeps): Readable<LayoutTier> {
  return readable<LayoutTier>(INITIAL_TIER, (set) => {
    let size: ViewportSize | null = null;
    let percent = 100;
    let current = INITIAL_TIER;

    const update = () => {
      if (!size) return;
      const next = layoutTierFor(size.width, size.height, (BASE_FONT_PX * percent) / 100);
      if (sameTier(next, current)) return;
      current = next;
      set(next);
    };

    const stopScale = textScale.subscribe((value) => {
      percent = value;
      update();
    });
    const stopViewport = viewport.subscribe((value) => {
      size = value;
      update();
    });
    return () => {
      stopScale();
      stopViewport();
    };
  });
}
