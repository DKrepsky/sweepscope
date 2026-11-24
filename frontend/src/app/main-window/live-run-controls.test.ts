import { get, writable, type Writable } from 'svelte/store';
import { describe, expect, it, vi } from 'vitest';
import type { AcquisitionStatus, InstrumentState } from '../../api';
import type { RecordingState } from '../../api/generated/RecordingState';
import { createCursorsStore } from '../cursors';
import type { InstrumentCommands } from '../instrument';
import {
  acquisitionStatus,
  connectionSnapshot,
  displayState,
  instrumentFacts,
  instrumentState,
  recordingState,
} from '../instrument/test-data';
import type { AnalyseCommands, RollInput } from '../views';
import { createLiveActions } from './live-actions';
import { createLiveMainWindow } from './live-main-window';

const inert = <T extends object>(names: string[]): T =>
  Object.fromEntries(names.map((name) => [name, vi.fn()])) as unknown as T;

function setup() {
  const status: Writable<AcquisitionStatus | undefined> = writable(acquisitionStatus());
  const instrument: Writable<InstrumentState | undefined> = writable(instrumentState());
  const recording: Writable<RecordingState | undefined> = writable(undefined);
  const analyseCommands = inert<AnalyseCommands>([
    'armSingle',
    'startAutoset',
    'setMath',
    'setPersistence',
    'setSpectrum',
    'startRecording',
    'stopRecording',
  ]);
  const commands = inert<InstrumentCommands>(['startRecording', 'setRunning', 'step', 'reset', 'setTrigger']);
  const window = createLiveMainWindow({
    stores: {
      connection: writable(connectionSnapshot()),
      instrument,
      status,
      facts: writable(instrumentFacts()),
      display: writable(displayState()),
      measurements: writable(undefined),
      zeroCalibration: writable(undefined),
    },
    analysis: {
      spectrum: writable(undefined),
      spectrumFrame: writable(undefined),
      recording,
      roll: writable<RollInput | undefined>(undefined),
    },
    analyseCommands,
    commands,
    calibration: {
      openCalibration: vi.fn(),
      startCalibration: vi.fn(),
      calibrateChannel: vi.fn(),
      closeCalibration: vi.fn(),
    },
    cursors: createCursorsStore(),
    pinned: { ...writable<readonly string[]>([]), set: vi.fn(() => Promise.resolve()), load: vi.fn() },
    files: inert(['saveSetup', 'loadSetup', 'exportCsv', 'exportRaw', 'saveScreenshot']),
    frames: writable(undefined),
    copyText: vi.fn(),
    notify: vi.fn(),
    renderer: 'Canvas 2D',
  });
  return { window, status, instrument, recording, analyseCommands, commands };
}

const autosetStatus = (running: boolean) => acquisitionStatus({ autoset: { running, noSignal: false, outcome: null } });

const failure = (id: number): RecordingState =>
  recordingState({ failure: { id, text: 'the disk is full', fileName: 'roll.ssroll', bytes: 1000 } });

const bannerIds = (window: ReturnType<typeof setup>['window']) => get(window).banners.map((banner) => banner.id);

describe('auto-setup request', () => {
  it('given an idle scope, when auto-setup is requested, then the command is sent', () => {
    const { window, analyseCommands } = setup();

    window.startAutoset();

    expect(analyseCommands.startAutoset).toHaveBeenCalledOnce();
  });

  it('given a run in progress, when auto-setup is requested again, then nothing is sent', () => {
    const { window, status, analyseCommands } = setup();
    status.set(autosetStatus(true));

    window.startAutoset();

    expect(analyseCommands.startAutoset).not.toHaveBeenCalled();
  });

  it('given a scope in roll mode, when auto-setup is requested by key, then nothing is sent', () => {
    const { window, instrument, analyseCommands } = setup();
    const base = instrumentState();
    const reason = 'Auto-setup needs a captured record; not available in roll mode.';
    instrument.set(instrumentState({ controls: { ...base.controls, autoset: { enabled: false, reason } } }));

    window.startAutoset();

    expect(analyseCommands.startAutoset).not.toHaveBeenCalled();
  });
});

describe('recording failure banner actions', () => {
  it('given a failure, when it is dismissed, then the banner goes and stays gone for the same id', () => {
    const { window, recording } = setup();
    recording.set(failure(1));
    expect(bannerIds(window)).toEqual(['recording']);

    window.bannerAction('dismiss');
    recording.set(failure(1));

    expect(bannerIds(window)).toEqual([]);
  });

  it('given a dismissed failure, when a newer one arrives, then the banner shows again', () => {
    const { window, recording } = setup();
    recording.set(failure(1));
    window.bannerAction('dismiss');

    recording.set(failure(2));

    expect(bannerIds(window)).toEqual(['recording']);
  });

  it('given a failure, when Choose new file is clicked, then the picker command runs and the banner goes', () => {
    const { window, recording, commands } = setup();
    recording.set(failure(1));

    window.bannerAction('choose-new-file');

    expect(commands.startRecording).toHaveBeenCalledOnce();
    expect(bannerIds(window)).toEqual([]);
  });
});

describe('banner actions on their own', () => {
  it('given a dismiss, when handled, then no command is sent', () => {
    const dismiss = vi.fn();
    const commands = inert<InstrumentCommands>(['startRecording', 'retryConnection', 'useSimulator']);
    const actions = createLiveActions({
      commands,
      calibration: {
        openCalibration: vi.fn(),
        startCalibration: vi.fn(),
        calibrateChannel: vi.fn(),
        closeCalibration: vi.fn(),
      },
      calibrationBlocked: () => null,
      read: () => ({ instrument: undefined, connection: undefined }),
      copyText: vi.fn(),
      notify: vi.fn(),
      select: vi.fn(),
      cursors: createCursorsStore(),
      canToggleCursors: () => true,
      pinned: { set: () => Promise.resolve() },
      files: inert(['saveSetup', 'loadSetup', 'exportCsv', 'exportRaw', 'saveScreenshot']),
      dismissRecordingFailure: dismiss,
    });

    actions.bannerAction('dismiss');

    expect(dismiss).toHaveBeenCalledOnce();
    expect(commands.startRecording).not.toHaveBeenCalled();
    expect(commands.retryConnection).not.toHaveBeenCalled();
  });
});
