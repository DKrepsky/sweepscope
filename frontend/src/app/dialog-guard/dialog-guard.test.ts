import { describe, expect, it } from 'vitest';
import { calibrationBlockedReason, modalBlockedReason } from './dialog-guard';

describe('modalBlockedReason', () => {
  it('AC-D1: given acquisition is running, when a modal is requested, then the reason is "Stop acquisition first"', () => {
    expect(modalBlockedReason(true)).toBe('Stop acquisition first');
  });

  it('AC-D1: given acquisition is stopped, when a modal is requested, then nothing blocks it', () => {
    expect(modalBlockedReason(false)).toBeNull();
  });
});

describe('calibrationBlockedReason', () => {
  it('given an unavailable tag, when the dialog is requested, then the tag reason blocks it', () => {
    expect(calibrationBlockedReason({ state: 'unavailable', reason: 'Stop recording first' })).toBe(
      'Stop recording first',
    );
  });

  it('given a scope that runs, when the calibration dialog is requested, then nothing blocks it', () => {
    expect(calibrationBlockedReason({ state: 'none', tooltip: { heading: '', lines: [] } })).toBeNull();
  });
});
