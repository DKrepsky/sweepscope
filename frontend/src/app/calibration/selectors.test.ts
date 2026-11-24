import { describe, expect, it } from 'vitest';
import type { ChannelState, InstrumentState, TooltipText, ZeroSession } from '../../api';
import { connectionSnapshot, instrumentState, recordingState, zeroCalibrationState } from '../instrument/test-data';
import { selectCalibrationDialog, selectCalibrationTag, selectPausedForCalibration } from './selectors';

const session = (patch: Partial<ZeroSession>): ZeroSession => ({
  step: 'intro',
  channel: null,
  doneChannels: [false, false, false, false],
  progress: null,
  notShorted: null,
  canContinue: false,
  overridden: [false, false, false, false],
  failure: null,
  table: null,
  ...patch,
});
const withSession = (patch: Partial<ZeroSession>) => zeroCalibrationState({ session: session(patch) });
const RANGES = 12;
const cells = (channel: number, label: string): [string, string, string] => [
  `${label} c${channel} x1`,
  `${label} c${channel} x10`,
  `${label} c${channel} x100`,
];
const tooltipText = (heading = 'Zero error'): TooltipText => ({
  heading,
  channels: [1, 2, 3, 4].map((channel) => ({
    cells: Array.from({ length: RANGES }, (_, index) => cells(channel, `r${index}`)),
  })) as TooltipText['channels'],
});
const unit = (calibrated: boolean, storedAtUtc: string | null = null) => ({
  calibrated,
  storedAtUtc,
  tooltip: tooltipText(),
});
const channelsOn = (...settings: Array<Partial<ChannelState>>): InstrumentState =>
  instrumentState({
    channels: instrumentState().channels.map((channel, index) => ({ ...channel, enabled: false, ...settings[index] })),
  });

describe('selectCalibrationTag', () => {
  it('given a ready scope without offsets, when selected, then the tag is none with the backend heading', () => {
    const tag = selectCalibrationTag(
      zeroCalibrationState({ unit: unit(false) }),
      connectionSnapshot(),
      undefined,
      undefined,
    );

    expect(tag).toEqual({ state: 'none', tooltip: { heading: 'Zero error', lines: [] } });
  });

  it('given stored offsets, when selected, then the tag is calibrated with the date formatted locally', () => {
    const stored = new Date(2026, 10, 24, 14, 5).toISOString();
    const zero = zeroCalibrationState({ unit: unit(true, stored) });

    expect(selectCalibrationTag(zero, connectionSnapshot(), undefined, undefined)).toMatchObject({
      state: 'calibrated',
      storedAt: '2026-11-24 14:05',
    });
  });

  it('given no slice yet, when selected, then the tag is unavailable with No device', () => {
    expect(selectCalibrationTag(undefined, connectionSnapshot(), undefined, undefined)).toEqual({
      state: 'unavailable',
      reason: 'No device',
    });
  });

  it('given a disconnected scope, when selected, then the tag is unavailable', () => {
    const connection = connectionSnapshot({ state: 'disconnected' });

    expect(selectCalibrationTag(zeroCalibrationState(), connection, undefined, undefined)).toMatchObject({
      state: 'unavailable',
    });
  });

  it('given the backend refuses with a reason, when selected, then the reason is shown', () => {
    const zero = zeroCalibrationState({ availability: { enabled: false, reason: 'Auto-setup is running' } });

    expect(selectCalibrationTag(zero, connectionSnapshot(), undefined, undefined)).toEqual({
      state: 'unavailable',
      reason: 'Auto-setup is running',
    });
  });

  it('given a recording is active, when selected, then the tag is unavailable with Stop recording first', () => {
    const tag = selectCalibrationTag(
      zeroCalibrationState(),
      connectionSnapshot(),
      recordingState({ active: true }),
      undefined,
    );

    expect(tag).toEqual({ state: 'unavailable', reason: 'Stop recording first' });
  });

  it('given a recording is active and the device is gone, when selected, then the device reason wins', () => {
    const zero = zeroCalibrationState({ availability: { enabled: false, reason: 'No device' } });

    expect(selectCalibrationTag(zero, connectionSnapshot(), recordingState({ active: true }), undefined)).toEqual({
      state: 'unavailable',
      reason: 'No device',
    });
  });
});

describe('selectCalibrationTag tooltip lines', () => {
  const lines = (instrument: InstrumentState, zero = zeroCalibrationState({ unit: unit(true) })) => {
    const tag = selectCalibrationTag(zero, connectionSnapshot(), undefined, instrument);
    return tag.state === 'unavailable' ? [] : tag.tooltip.lines;
  };
  const on = (index: number, probe = 1): Partial<ChannelState> => {
    const base = instrumentState().channels[0] as ChannelState;
    return { enabled: true, probe, range: { ...base.range, index } };
  };

  it.each([true, false])(
    'given channels at the first and the last range, when calibrated is %s, then two lines show those ranges',
    (calibrated) => {
      const zero = zeroCalibrationState({ unit: unit(calibrated) });

      expect(lines(channelsOn(on(0), on(11)), zero)).toEqual(['CH1 r0 c1 x1', 'CH2 r11 c2 x1']);
    },
  );

  it('given a range knob turn, when the instrument slice changes, then the line changes without a new calibration slice', () => {
    const zero = zeroCalibrationState({ unit: unit(true) });

    const before = lines(channelsOn(on(3)), zero);
    const after = lines(channelsOn(on(4)), zero);

    expect([before, after]).toEqual([['CH1 r3 c1 x1'], ['CH1 r4 c1 x1']]);
  });

  it.each([
    [10, 'CH1 r2 c1 x10'],
    [100, 'CH1 r2 c1 x100'],
  ])('given a %ix probe, when selected, then the text for that probe shows', (probe, expected) => {
    expect(lines(channelsOn(on(2, probe)))).toEqual([expected]);
  });

  it('given an unknown probe factor, when selected, then the channel line is hidden', () => {
    expect(lines(channelsOn(on(2, 7)))).toEqual([]);
  });

  it('given a disabled channel, when selected, then it has no line', () => {
    expect(lines(channelsOn(on(0), { ...on(1), enabled: false }, on(2)))).toEqual(['CH1 r0 c1 x1', 'CH3 r2 c3 x1']);
  });

  it('given no enabled channel, when selected, then the heading shows alone', () => {
    const zero = zeroCalibrationState({ unit: unit(true) });

    const tag = selectCalibrationTag(zero, connectionSnapshot(), undefined, channelsOn());

    expect(tag).toMatchObject({ tooltip: { heading: 'Zero error', lines: [] } });
  });

  it('given a range index the backend has no text for, when selected, then that line is hidden', () => {
    expect(lines(channelsOn(on(99)))).toEqual([]);
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

  it('given a rejection that can continue, when selected, then the dialog offers it and lists overridden channels', () => {
    const overridden: ZeroSession['overridden'] = [true, false, false, false];

    const dialog = selectCalibrationDialog(
      withSession({ step: 'notShorted', channel: 2, canContinue: true, overridden }),
    );

    expect(dialog).toMatchObject({ canContinue: true, overridden });
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
