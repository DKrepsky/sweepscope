import { describe, expect, it } from 'vitest';
import { iconNames, icons } from './index';

const designSet = [
  'play',
  'stop',
  'single',
  'force',
  'autoset',
  'cursors',
  'math',
  'reference',
  'save',
  'screenshot',
  'settings',
  'minus',
  'plus',
  'chevron',
  'rising-edge',
  'falling-edge',
  'warning',
  'info',
  'close',
  'ok',
  'disconnected',
  'record',
  'pulse-positive',
  'pulse-negative',
  'peak',
];

describe('icons', () => {
  it('given the icon set, when names are listed, then they are the design set including the M3 icons', () => {
    expect([...iconNames].sort()).toEqual([...designSet].sort());
  });

  it('given every icon, when its shapes are read, then each has a path on the 16 px grid', () => {
    for (const name of iconNames) {
      expect(icons[name].viewBox).toBe('0 0 16 16');
      expect(icons[name].shapes.length).toBeGreaterThan(0);
      expect(icons[name].shapes.every((shape) => shape.d.startsWith('M'))).toBe(true);
    }
  });
});
