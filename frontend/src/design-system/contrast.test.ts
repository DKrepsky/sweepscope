import { describe, expect, it } from 'vitest';
import { minimumPairwiseDistance, type Deficiency } from './colour-blind';
import { contrastRatio } from './contrast';
import { tokens } from './index';

const colour = (name: string) => tokens[`color-${name}` as keyof typeof tokens];
const channels = ['ch1', 'ch2', 'ch3', 'ch4'].map(colour);
const surfaces = ['bg-panel', 'bg-app', 'bg-inset'];

describe('contrast (AC-T2)', () => {
  it.each(['text-primary', 'text-secondary', 'text-muted'].flatMap((text) => surfaces.map((bg) => [text, bg])))(
    'given %s on %s, when contrast is computed, then it is at least 4.5',
    (text, bg) => {
      expect(contrastRatio(colour(text!), colour(bg!))).toBeGreaterThanOrEqual(4.5);
    },
  );

  it.each(['focus', 'accent'])('given %s on bg-panel, when contrast is computed, then it is at least 3', (name) => {
    expect(contrastRatio(colour(name), colour('bg-panel'))).toBeGreaterThanOrEqual(3);
  });

  it.each(['bg-panel', 'bg-inset', 'bg-raised', 'bg-hover'])(
    'given border-control on %s, when contrast is computed, then it is at least 3',
    (bg) => {
      expect(contrastRatio(colour('border-control'), colour(bg))).toBeGreaterThanOrEqual(3);
    },
  );

  it('given the decorative borders on bg-panel, when contrast is computed, then they are below 3 because they are exempt', () => {
    for (const name of ['border-subtle', 'border', 'border-strong']) {
      expect(contrastRatio(colour(name), colour('bg-panel'))).toBeLessThan(3);
    }
  });

  it.each(['ch1', 'ch2', 'ch3', 'ch4'])(
    'given channel colour %s on bg-scope, when contrast is computed, then it is at least 6.5',
    (name) => {
      expect(contrastRatio(colour(name), colour('bg-scope'))).toBeGreaterThanOrEqual(6.5);
    },
  );
});

describe('colour-blind distinctness (AC-T3)', () => {
  it.each(['deuteranopia', 'protanopia', 'tritanopia'] as Deficiency[])(
    'given the four channel colours under %s, when pairwise CIELAB distance is computed, then every pair is at least 20',
    (deficiency) => {
      const minimum = minimumPairwiseDistance(channels, deficiency);
      console.info(`minimum channel distance under ${deficiency}: ${minimum.toFixed(1)}`);
      expect(minimum).toBeGreaterThanOrEqual(20);
    },
  );

  it('given the four channel colours under normal vision, when pairwise CIELAB distance is computed, then the minimum is at least 34', () => {
    const minimum = minimumPairwiseDistance(channels, 'normal');
    console.info(`minimum channel distance under normal vision: ${minimum.toFixed(1)}`);
    expect(minimum).toBeGreaterThanOrEqual(34);
  });
});
