import { readable, type Readable } from 'svelte/store';

export type WidthTier = 'w1' | 'w2' | 'w3' | 'scroll';
/** h4 is design 3.2 step 7 (dock groups as tabs); scroll is a window under the content floor. */
export type HeightTier = 'h1' | 'h2' | 'h3' | 'h4' | 'scroll';

export interface LayoutTier {
  width: WidthTier;
  height: HeightTier;
  /** Computed root font size in px: 1 rem. Layout code converts rem to px with it. */
  rootFontPx: number;
}

export interface PixelSize {
  width: number;
  height: number;
}

/** Design 3.3: the scope screen never gets smaller than this, whatever the text scale. */
export const SCREEN_MINIMUM_REM = { width: 32, height: 25.6 } as const;

export function screenMinimumPx(rootFontPx: number): PixelSize {
  return {
    width: Math.ceil(SCREEN_MINIMUM_REM.width * rootFontPx),
    height: Math.ceil(SCREEN_MINIMUM_REM.height * rootFontPx),
  };
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
/**
 * Minimum window heights in rem per collapse step, found by measurement (layout-calibration.md). The narrow column
 * (W3 and below) has a two-row dock and a two-line status bar, so every step needs more height.
 */
export const HEIGHT_TIER_REM = {
  wide: [
    [47.25, 'h1'],
    [46.25, 'h2'],
    [45, 'h4'],
  ],
  narrow: [
    [59, 'h1'],
    [57, 'h2'],
    [50.25, 'h3'],
    [45, 'h4'],
  ],
} as const satisfies Record<string, readonly (readonly [number, HeightTier])[]>;

function pick<T>(rem: number, tiers: readonly (readonly [number, T])[], fallback: T): T {
  return tiers.find(([minimum]) => rem >= minimum)?.[1] ?? fallback;
}

export function layoutTierFor(widthPx: number, heightPx: number, rootFontPx: number): LayoutTier {
  const width = pick(widthPx / rootFontPx, WIDTH_TIERS, 'scroll');
  const heights = width === 'w1' || width === 'w2' ? HEIGHT_TIER_REM.wide : HEIGHT_TIER_REM.narrow;
  return { width, height: pick(heightPx / rootFontPx, heights, 'scroll'), rootFontPx };
}

export const INITIAL_TIER: LayoutTier = { width: 'w1', height: 'h1', rootFontPx: BASE_FONT_PX };

const sameTier = (a: LayoutTier, b: LayoutTier) =>
  a.width === b.width && a.height === b.height && a.rootFontPx === b.rootFontPx;

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
