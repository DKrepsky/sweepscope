import type { HeightTier } from './layout-tier';

/** What each height step takes away, in the order of design 3.2 (steps 4 to 7); `scroll` keeps every step applied. */
export interface HeightCollapse {
  recordOverview: boolean;
  smallRows: boolean;
  channelTabs: boolean;
  essentialStatusOnly: boolean;
  groupTabs: boolean;
}

const STEPS: Record<HeightTier, number> = { h1: 0, h2: 1, h3: 2, h4: 3, scroll: 3 };

export function heightCollapse(height: HeightTier): HeightCollapse {
  const step = STEPS[height];
  return {
    recordOverview: step === 0,
    smallRows: step >= 1,
    channelTabs: step >= 2,
    essentialStatusOnly: step >= 2,
    groupTabs: step >= 3,
  };
}
