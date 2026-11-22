import { describe, expect, it } from 'vitest';
import { modalBlockedReason } from './dialog-guard';

describe('modalBlockedReason', () => {
  it('AC-D1: given acquisition is running, when a modal is requested, then the reason is "Stop acquisition first"', () => {
    expect(modalBlockedReason(true)).toBe('Stop acquisition first');
  });

  it('AC-D1: given acquisition is stopped, when a modal is requested, then nothing blocks it', () => {
    expect(modalBlockedReason(false)).toBeNull();
  });
});
