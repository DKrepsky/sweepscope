import { describe, expect, it } from 'vitest';
import { recordingState } from '../instrument/test-data';
import { createFailureDismissal } from './failure-dismissal';
import { failureText, formatBytes, formatElapsed, recordingChip } from './recording-text';
import { get } from 'svelte/store';

describe('formatElapsed', () => {
  it.each([
    [0, '00:00:00'],
    [3, '00:00:03'],
    [83.9, '00:01:23'],
    [5025, '01:23:45'],
    [-4, '00:00:00'],
  ])('given %s seconds, when formatted, then it reads %s', (seconds, text) => {
    expect(formatElapsed(seconds)).toBe(text);
  });
});

describe('formatBytes', () => {
  it.each([
    [0, '0 B'],
    [999, '999 B'],
    [12_400_000, '11.8 MiB'],
    [1_200_000_000, '1.12 GiB'],
    [120_000, '117 KiB'],
  ])('given %s bytes, when formatted, then it reads %s', (bytes, text) => {
    expect(formatBytes(bytes)).toBe(text);
  });
});

describe('recordingChip', () => {
  it('given an active recording, when the chip is built, then it shows elapsed time and size from the slice', () => {
    const state = recordingState({
      active: true,
      duration: { value: 3, text: '3', unit: 's' },
      bytesWritten: { value: 12288, text: '12.0', unit: 'KiB' },
    });

    expect(recordingChip(state)).toBe('Rec 00:00:03 · 12.0 KiB');
  });

  it('given no recording, when the chip is built, then there is none', () => {
    expect(recordingChip(recordingState({ active: false }))).toBeUndefined();
    expect(recordingChip(undefined)).toBeUndefined();
  });
});

describe('failureText', () => {
  it('given a disk-full failure, when the text is built, then it names the reason, the size, the file and that the display runs', () => {
    const text = failureText({
      id: 1,
      text: 'the disk is full',
      fileName: 'roll-2026-11-24.ssroll',
      bytes: 1_200_000_000,
    });

    expect(text).toBe(
      'Recording stopped: the disk is full. 1.12 GiB written to roll-2026-11-24.ssroll. The display is still running.',
    );
  });
});

describe('failure dismissal', () => {
  it('given a dismissed failure, when an older or the same id is checked, then it stays dismissed and a newer one is not', () => {
    const dismissal = createFailureDismissal();

    dismissal.dismiss(2);
    dismissal.dismiss(1);

    expect(get(dismissal)).toBe(2);
  });
});
