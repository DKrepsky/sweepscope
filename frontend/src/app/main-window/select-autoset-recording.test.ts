import { describe, expect, it } from 'vitest';
import { acquisitionStatus, instrumentState, recordingState } from '../instrument/test-data';
import { selectAutoset } from './select-autoset';
import { selectBanners } from './select-banners';
import { selectRecording } from './select-recording';
import { modelInputs } from './test-inputs';

const running = (patch: Partial<ReturnType<typeof acquisitionStatus>['autoset']>) =>
  acquisitionStatus({ autoset: { running: false, noSignal: false, outcome: null, ...patch } });

describe('selectAutoset', () => {
  it('given a run in progress, when selected, then it is busy and not available with the reason', () => {
    const model = selectAutoset(modelInputs({ status: running({ running: true }) }));

    expect(model).toMatchObject({ busy: true, available: { enabled: false, reason: 'Auto-setup is running' } });
  });

  it('given an idle scope, when selected, then it is available and not busy', () => {
    expect(selectAutoset(modelInputs())).toEqual({ busy: false, noSignal: false, available: { enabled: true } });
  });

  it('given the backend flags no signal, when selected, then the model carries it until it clears', () => {
    expect(selectAutoset(modelInputs({ status: running({ noSignal: true }) })).noSignal).toBe(true);
    expect(selectAutoset(modelInputs({ status: running({ noSignal: false }) })).noSignal).toBe(false);
  });

  it('given no enabled channel, when selected, then the backend reason is shown', () => {
    const base = instrumentState();
    const controls = { ...base.controls, autoset: { enabled: false, reason: 'Enable a channel first' } };

    const model = selectAutoset(modelInputs({ instrument: instrumentState({ controls }) }));

    expect(model.available).toEqual({ enabled: false, reason: 'Enable a channel first' });
  });
});

describe('selectRecording', () => {
  it('given an active recording, when selected, then the chip reads elapsed time and size', () => {
    const recording = recordingState({
      active: true,
      duration: { value: 3, text: '3', unit: 's' },
      bytesWritten: { value: 12000, text: '12', unit: 'kB' },
    });

    expect(selectRecording(modelInputs({ recording })).chip).toBe('Rec 00:00:03 · 12 kB');
  });

  it('given no recording, when selected, then there is no chip', () => {
    expect(selectRecording(modelInputs({ recording: recordingState() })).chip).toBeUndefined();
  });

  it('given a scope outside roll, when selected, then recording carries the backend reason', () => {
    const base = instrumentState();
    const reason = 'Recording needs roll mode (500 ms/div or slower)';
    const controls = { ...base.controls, record: { enabled: false, reason } };

    const model = selectRecording(modelInputs({ instrument: instrumentState({ controls }) }));

    expect(model.available).toEqual({ enabled: false, reason });
  });
});

describe('recording failure banner', () => {
  const failed = recordingState({
    failure: { id: 2, text: 'the disk is full', fileName: 'roll.ssroll', bytes: 1_200_000_000 },
  });

  it('given a failure, when selected, then one error banner offers Choose new file and Dismiss', () => {
    const banners = selectBanners(modelInputs({ recording: failed }));

    expect(banners).toEqual([
      {
        id: 'recording',
        kind: 'error',
        text: 'Recording stopped: the disk is full. 1.20 GB written to roll.ssroll. The display is still running.',
        actions: [
          { id: 'choose-new-file', label: 'Choose new file' },
          { id: 'dismiss', label: 'Dismiss' },
        ],
      },
    ]);
  });

  it('given the failure was dismissed, when selected, then there is no banner for the same id', () => {
    expect(selectBanners(modelInputs({ recording: failed, dismissedRecordingFailure: 2 }))).toEqual([]);
  });

  it('given a newer failure than the dismissed one, when selected, then the banner shows again', () => {
    expect(selectBanners(modelInputs({ recording: failed, dismissedRecordingFailure: 1 }))).toHaveLength(1);
  });

  it('given a connection error and a failure, when selected, then both banners show', () => {
    const inputs = modelInputs({
      recording: failed,
      connection: { ...modelInputs().connection!, state: 'disconnected' },
    });

    expect(selectBanners(inputs).map((banner) => banner.id)).toEqual(['connection', 'recording']);
  });
});
