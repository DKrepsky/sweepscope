import { describe, expect, it } from 'vitest';
import type { ZeroSession } from '../../api';
import { connectionSnapshot, recordingState, zeroCalibrationState } from '../instrument/test-data';
import { selectCalibrationDialog, selectCalibrationTag, selectPausedForCalibration } from './selectors';

const session = (patch: Partial<ZeroSession>): ZeroSession => ({
  step: 'intro',
  channel: null,
  doneChannels: [false, false, false, false],
  progress: null,
  notShorted: null,
  failure: null,
  table: null,
  ...patch,
});
const withSession = (patch: Partial<ZeroSession>) => zeroCalibrationState({ session: session(patch) });
const unit = (calibrated: boolean, storedAtUtc: string | null = null) => ({
  calibrated,
  storedAtUtc,
  tooltip: { rows: [{ label: 'CH1', value: '+0.66 mV' }], notes: ['a note'] },
});

describe('selectCalibrationTag', () => {
  it('given a ready scope without offsets, when selected, then the tag is none with the backend tooltip', () => {
    const tag = selectCalibrationTag(zeroCalibrationState({ unit: unit(false) }), connectionSnapshot(), undefined);

    expect(tag).toEqual({ state: 'none', tooltip: unit(false).tooltip });
  });

  it('given stored offsets, when selected, then the tag is calibrated with the date formatted locally', () => {
    const stored = new Date(2026, 10, 24, 14, 5).toISOString();
    const zero = zeroCalibrationState({ unit: unit(true, stored) });

    expect(selectCalibrationTag(zero, connectionSnapshot(), undefined)).toMatchObject({
      state: 'calibrated',
      storedAt: '2026-11-24 14:05',
    });
  });

  it('given no slice yet, when selected, then the tag is unavailable with No device', () => {
    expect(selectCalibrationTag(undefined, connectionSnapshot(), undefined)).toEqual({
      state: 'unavailable',
      reason: 'No device',
    });
  });

  it('given a disconnected scope, when selected, then the tag is unavailable', () => {
    const connection = connectionSnapshot({ state: 'disconnected' });

    expect(selectCalibrationTag(zeroCalibrationState(), connection, undefined)).toMatchObject({
      state: 'unavailable',
    });
  });

  it('given the backend refuses with a reason, when selected, then the reason is shown', () => {
    const zero = zeroCalibrationState({ availability: { enabled: false, reason: 'Auto-setup is running' } });

    expect(selectCalibrationTag(zero, connectionSnapshot(), undefined)).toEqual({
      state: 'unavailable',
      reason: 'Auto-setup is running',
    });
  });

  it('given a recording is active, when selected, then the tag is unavailable with Stop recording first', () => {
    const tag = selectCalibrationTag(zeroCalibrationState(), connectionSnapshot(), recordingState({ active: true }));

    expect(tag).toEqual({ state: 'unavailable', reason: 'Stop recording first' });
  });

  it('given a recording is active and the device is gone, when selected, then the device reason wins', () => {
    const zero = zeroCalibrationState({ availability: { enabled: false, reason: 'No device' } });

    expect(selectCalibrationTag(zero, connectionSnapshot(), recordingState({ active: true }))).toEqual({
      state: 'unavailable',
      reason: 'No device',
    });
  });
});

describe('selectCalibrationDialog', () => {
  it('given no session, when selected, then there is no dialog', () => {
    expect(selectCalibrationDialog(zeroCalibrationState())).toBeUndefined();
  });

  it.each([
    ['intro', null, 1],
    ['channel', 1, 2],
    ['channel', 4, 5],
    ['measuring', 2, 3],
    ['notShorted', 3, 4],
    ['done', null, 6],
    ['failed', 2, 3],
    ['failed', null, 1],
  ] as const)('given step %s on channel %s, when selected, then it is step %i of 6', (step, channel, number) => {
    expect(selectCalibrationDialog(withSession({ step, channel }))?.stepNumber).toBe(number);
  });

  it('given a done session, when selected, then table and finished channels pass through', () => {
    const doneChannels: ZeroSession['doneChannels'] = [true, true, true, true];

    const dialog = selectCalibrationDialog(withSession({ step: 'done', doneChannels, table: [] }));

    expect(dialog).toMatchObject({ doneChannels, table: [], progress: null });
  });

  it('given a calibrated unit, when its dialog opens, then the stored date is carried for the intro note', () => {
    const stored = new Date(2026, 0, 2, 3, 4).toISOString();
    const zero = zeroCalibrationState({ unit: unit(true, stored), session: session({}) });

    expect(selectCalibrationDialog(zero)).toMatchObject({ calibrated: true, storedAt: '2026-01-02 03:04' });
  });

  it('given a not shorted step, when selected, then the backend lines are listed', () => {
    const dialog = selectCalibrationDialog(withSession({ step: 'notShorted', channel: 1, notShorted: ['line'] }));

    expect(dialog?.notShorted).toEqual(['line']);
  });
});

describe('selectPausedForCalibration', () => {
  it('given a session in any step, when selected, then acquisition is paused', () => {
    expect(selectPausedForCalibration(withSession({ step: 'done' }))).toBe(true);
  });

  it('given no session or no slice, when selected, then acquisition is not paused', () => {
    expect([selectPausedForCalibration(zeroCalibrationState()), selectPausedForCalibration(undefined)]).toEqual([
      false,
      false,
    ]);
  });
});
