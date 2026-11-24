import { describe, expect, it } from 'vitest';
import { formatDbv, formatFrequency } from './index';

describe('formatFrequency', () => {
  it('given 1001 Hz, when formatted, then it reads 1.001 kHz', () => {
    expect(formatFrequency(1001)).toBe('1.001 kHz');
  });

  it('given 12.21 Hz, when formatted, then four significant digits are kept', () => {
    expect(formatFrequency(12.2070312)).toBe('12.21 Hz');
  });

  it('given a value that is not finite, when formatted, then it shows the no-value dash', () => {
    expect(formatFrequency(Number.NaN)).toBe('- Hz');
  });
});

describe('formatDbv', () => {
  it('given -6.94 dB, when formatted, then it reads with a true minus and dBV', () => {
    expect(formatDbv(-6.94)).toBe('−6.9 dBV');
  });

  it('given a tiny negative level, when formatted, then no negative zero is shown', () => {
    expect(formatDbv(-0.04)).toBe('0.0 dBV');
  });

  it('given the top of the axis, when formatted, then it reads 10.0 dBV', () => {
    expect(formatDbv(10)).toBe('10.0 dBV');
  });
});
