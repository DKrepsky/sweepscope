import { describe, expect, it } from 'vitest';
import { formatStoredDate } from './format-date';

describe('formatStoredDate', () => {
  it('given an ISO instant, when formatted, then the local date and minutes show', () => {
    expect(formatStoredDate(new Date(2026, 10, 4, 9, 7).toISOString())).toBe('2026-11-04 09:07');
  });

  it.each([null, undefined, '', 'yesterday'])('given %s, when formatted, then there is no date', (text) => {
    expect(formatStoredDate(text)).toBeUndefined();
  });
});
