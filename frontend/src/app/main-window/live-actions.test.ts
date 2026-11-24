import { describe, expect, it, vi } from 'vitest';
import type { InstrumentCommands } from '../instrument';
import type { InstrumentState } from '../../api';
import { connectionSnapshot, instrumentState } from '../instrument/test-data';
import { createLiveActions } from './live-actions';

function setup(
  state: InstrumentState | undefined = instrumentState(),
  connection = connectionSnapshot(),
  { noInstrument = false, canToggleCursors = true } = {},
) {
  const commands: InstrumentCommands = {
    setRunning: vi.fn(),
    forceTrigger: vi.fn(),
    setChannel: vi.fn(),
    step: vi.fn(),
    reset: vi.fn(),
    setTriggerPosition: vi.fn(),
    setTriggerLevel: vi.fn(),
    setTrigger: vi.fn(),
    useSimulator: vi.fn(),
    retryConnection: vi.fn(),
    setInterpolation: vi.fn(),
    armSingle: vi.fn(),
    startAutoset: vi.fn(),
    setMath: vi.fn(),
    setPersistence: vi.fn(),
    setSpectrum: vi.fn(),
    startRecording: vi.fn(),
    stopRecording: vi.fn(),
  };
  const copyText = vi.fn();
  const notify = vi.fn();
  const select = vi.fn();
  const cursors = { toggle: vi.fn(), move: vi.fn(), remove: vi.fn(), clear: vi.fn() };
  const pinned = { set: vi.fn(() => Promise.resolve()) };
  const files = {
    saveSetup: vi.fn(() => Promise.resolve()),
    loadSetup: vi.fn(() => Promise.resolve()),
    exportCsv: vi.fn(() => Promise.resolve()),
    exportRaw: vi.fn(() => Promise.resolve()),
    saveScreenshot: vi.fn(() => Promise.resolve()),
  };
  const actions = createLiveActions({
    commands,
    read: () => ({ instrument: noInstrument ? undefined : state, connection }),
    copyText,
    notify,
    select,
    cursors,
    canToggleCursors: () => canToggleCursors,
    pinned,
    files,
    dismissRecordingFailure: vi.fn(),
  });
  return { actions, commands, copyText, notify, select, cursors, pinned, files };
}

const withChannel = (number: number, patch: Partial<InstrumentState['channels'][number]>): InstrumentState => {
  const state = instrumentState();
  return { ...state, channels: state.channels.map((c) => (c.number === number ? { ...c, ...patch } : c)) };
};

describe('live actions: run', () => {
  it('given a running instrument, when Run is toggled, then it stops', () => {
    const { actions, commands } = setup(instrumentState({ running: true }));
    actions.toggleRun();
    expect(commands.setRunning).toHaveBeenCalledWith(false);
  });

  it('given a stopped instrument, when Run is toggled, then it runs', () => {
    const { actions, commands } = setup(instrumentState({ running: false }));
    actions.toggleRun();
    expect(commands.setRunning).toHaveBeenCalledWith(true);
  });

  it('given a force request, when called, then the command is sent', () => {
    const { actions, commands } = setup();
    actions.forceTrigger();
    expect(commands.forceTrigger).toHaveBeenCalledOnce();
  });
});

describe('live actions: channels', () => {
  it('given a channel, when enabled or disabled, then the patch carries enabled', () => {
    const { actions, commands } = setup();
    actions.setChannelEnabled(2, true);
    expect(commands.setChannel).toHaveBeenCalledWith(2, { enabled: true });
  });

  it('given a range step, when stepped faster and slower, then the range target and direction are sent', () => {
    const { actions, commands } = setup();
    actions.stepVoltsPerDiv(3, -1);
    actions.stepVoltsPerDiv(3, 1);
    expect(commands.step).toHaveBeenNthCalledWith(1, { kind: 'range', channel: 3 }, -1);
    expect(commands.step).toHaveBeenNthCalledWith(2, { kind: 'range', channel: 3 }, 1);
  });

  it('given a range reset, when called, then the range target is reset', () => {
    const { actions, commands } = setup();
    actions.resetVoltsPerDiv(4);
    expect(commands.reset).toHaveBeenCalledWith({ kind: 'range', channel: 4 });
  });

  it('given a drag, when the position changes, then the divisions are sent', () => {
    const { actions, commands } = setup();
    actions.setChannelPosition(1, -2.5);
    expect(commands.setChannel).toHaveBeenCalledWith(1, { positionDivs: -2.5 });
  });

  it.each([
    ['2.5', 2.5],
    ['−2.00', -2],
    ['+3', 3],
    ['1,5', 1.5],
    [' 0.5 div ', 0.5],
  ])('given the text %s, when the position is committed, then %s divisions are sent', (text, divisions) => {
    const { actions, commands } = setup();
    actions.commitChannelPosition(1, text);
    expect(commands.setChannel).toHaveBeenCalledWith(1, { positionDivs: divisions });
  });

  it('given text that is not a number, when the position is committed, then nothing is sent and an error is shown', () => {
    const { actions, commands, notify } = setup();
    actions.commitChannelPosition(1, 'abc');
    expect(commands.setChannel).not.toHaveBeenCalled();
    expect(notify).toHaveBeenCalledWith('error', expect.stringContaining('abc'));
  });

  it('given coupling, bandwidth and invert, when changed, then the patches carry the new values', () => {
    const { actions, commands } = setup(withChannel(1, { bandwidthLimit: true, invert: false }));
    actions.setCoupling(1, 'ac');
    actions.toggleBandwidth(1);
    actions.toggleInvert(1);
    expect(commands.setChannel).toHaveBeenNthCalledWith(1, 1, { coupling: 'ac' });
    expect(commands.setChannel).toHaveBeenNthCalledWith(2, 1, { bandwidthLimit: false });
    expect(commands.setChannel).toHaveBeenNthCalledWith(3, 1, { invert: true });
  });

  it.each([
    [1, 10],
    [10, 100],
    [100, 1],
  ])('given a probe of %sx, when cycled, then %sx is requested', (probe, next) => {
    const { actions, commands } = setup(withChannel(1, { probe }));
    actions.cycleProbe(1);
    expect(commands.setChannel).toHaveBeenCalledWith(1, { probe: next });
  });

  it('given a channel selection, when called, then the presentation store is told and no command is sent', () => {
    const { actions, commands, select } = setup();
    actions.selectChannel(4);
    expect(select).toHaveBeenCalledWith(4);
    expect(commands.setChannel).not.toHaveBeenCalled();
  });
});

describe('live actions: horizontal and trigger', () => {
  it('given the time base, when stepped and reset, then the timeBase target is used', () => {
    const { actions, commands } = setup();
    actions.stepTimePerDiv(1);
    actions.resetTimePerDiv();
    expect(commands.step).toHaveBeenCalledWith({ kind: 'timeBase' }, 1);
    expect(commands.reset).toHaveBeenCalledWith({ kind: 'timeBase' });
  });

  it('given the trigger position, when set and stepped, then the fraction and the target are sent', () => {
    const { actions, commands } = setup();
    actions.setTriggerPosition(0.3);
    actions.stepTriggerPosition(-1);
    expect(commands.setTriggerPosition).toHaveBeenCalledWith(0.3);
    expect(commands.step).toHaveBeenCalledWith({ kind: 'triggerPosition' }, -1);
  });

  it('given the trigger level, when set, stepped and reset, then divisions and targets are sent', () => {
    const { actions, commands } = setup();
    actions.setTriggerLevel(1.5);
    actions.stepTriggerLevel(1);
    actions.resetTriggerLevel();
    expect(commands.setTriggerLevel).toHaveBeenCalledWith(1.5);
    expect(commands.step).toHaveBeenCalledWith({ kind: 'triggerLevel' }, 1);
    expect(commands.reset).toHaveBeenCalledWith({ kind: 'triggerLevel' });
  });

  it('given source, slope and sweep, when changed, then each is a trigger patch', () => {
    const { actions, commands } = setup();
    actions.setTriggerSource(2);
    actions.setTriggerSlope('falling');
    actions.setTriggerSweep('normal');
    expect(commands.setTrigger).toHaveBeenNthCalledWith(1, { source: 2 });
    expect(commands.setTrigger).toHaveBeenNthCalledWith(2, { slope: 'falling' });
    expect(commands.setTrigger).toHaveBeenNthCalledWith(3, { sweep: 'normal' });
  });

  it('given an interpolation choice, when set, then the backend command is sent with the mode', () => {
    const { actions, commands } = setup();
    actions.setInterpolation('dots');
    expect(commands.setInterpolation).toHaveBeenCalledExactlyOnceWith('dots');
  });
});

describe('live actions: cursors and pinned measurements', () => {
  it('given a toggle, when called, then the cursors store toggles', () => {
    const { actions, cursors } = setup();
    actions.toggleCursors();
    expect(cursors.toggle).toHaveBeenCalledOnce();
  });

  it('given no waveform to measure, when cursors are toggled, then the store is not touched', () => {
    const { actions, cursors } = setup(undefined, undefined, { canToggleCursors: false });
    actions.toggleCursors();
    expect(cursors.toggle).not.toHaveBeenCalled();
  });

  it('given a cursor move, when called, then the store gets the id and position', () => {
    const { actions, cursors } = setup();
    actions.moveCursor('y2', 0.4);
    expect(cursors.move).toHaveBeenCalledExactlyOnceWith('y2', 0.4);
  });

  it('given a cursor removal, when called, then the store removes that cursor', () => {
    const { actions, cursors } = setup();
    actions.removeCursor('x1');
    expect(cursors.remove).toHaveBeenCalledExactlyOnceWith('x1');
  });

  it('given a clear request, when called, then the store clears', () => {
    const { actions, cursors } = setup();
    actions.clearCursors();
    expect(cursors.clear).toHaveBeenCalledOnce();
  });

  it('given a pin request, when called, then the choice is stored', () => {
    const { actions, pinned } = setup();
    actions.setMeasurementPinned('vpp', false);
    expect(pinned.set).toHaveBeenCalledExactlyOnceWith('vpp', false);
  });
});

describe('live actions: files', () => {
  it.each([
    ['saveSetup', 'saveSetup'],
    ['loadSetup', 'loadSetup'],
    ['exportCsv', 'exportCsv'],
    ['exportRaw', 'exportRaw'],
    ['saveScreenshot', 'saveScreenshot'],
  ] as const)('given %s, when called, then the matching file action runs', (name, expected) => {
    const { actions, files } = setup();
    actions[name]();
    expect(files[expected]).toHaveBeenCalledOnce();
  });

  it('given a file action that rejects, when called, then an error is notified', async () => {
    const { actions, files, notify } = setup();
    files.exportCsv.mockRejectedValueOnce(new Error('boom'));
    actions.exportCsv();
    await Promise.resolve();
    await Promise.resolve();
    expect(notify).toHaveBeenCalledWith('error', expect.any(String));
  });
});

describe('live actions: banners and clipboard', () => {
  it('given the retry action, when called, then the connection is retried', () => {
    const { actions, commands } = setup();
    actions.bannerAction('retry');
    expect(commands.retryConnection).toHaveBeenCalledOnce();
  });

  it('given the use-simulator action, when called, then the simulator is enabled', () => {
    const { actions, commands } = setup();
    actions.bannerAction('use-simulator');
    expect(commands.useSimulator).toHaveBeenCalledWith(true);
  });

  it('given a permission issue, when copy-command is called, then the install command is copied', () => {
    const connection = connectionSnapshot({
      state: 'error',
      issue: {
        kind: 'permissionDenied',
        message: 'x',
        help: { ruleFile: 'r', installCommand: 'sudo cp r /etc/udev/rules.d/', doc: 'd' },
      },
    });
    const { actions, copyText } = setup(instrumentState(), connection);
    actions.bannerAction('copy-command');
    expect(copyText).toHaveBeenCalledWith('sudo cp r /etc/udev/rules.d/');
  });

  it('given no help in the connection, when copy-command is called, then nothing is copied', () => {
    const { actions, copyText } = setup();
    actions.bannerAction('copy-command');
    expect(copyText).not.toHaveBeenCalled();
  });

  it('given a text, when copied, then the clipboard dependency receives it', () => {
    const { actions, copyText } = setup();
    actions.copyText('hello');
    expect(copyText).toHaveBeenCalledWith('hello');
  });
});

describe('live actions: no instrument state', () => {
  it('given no instrument state, when a toggle is requested, then nothing is sent', () => {
    const { actions, commands } = setup(undefined, undefined, { noInstrument: true });
    actions.toggleRun();
    actions.toggleBandwidth(1);
    actions.cycleProbe(1);
    actions.toggleInvert(1);
    expect(commands.setRunning).not.toHaveBeenCalled();
    expect(commands.setChannel).not.toHaveBeenCalled();
  });
});
