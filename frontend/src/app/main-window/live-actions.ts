import type { ChannelPatch, ConnectionSnapshot, InstrumentState } from '../../api';
import type { CalibrationCommands } from '../calibration';
import type { CursorsStore } from '../cursors';
import type { FileActions } from '../files';
import type { InstrumentCommands } from '../instrument';
import type { MainWindowActions } from './actions';
import type { AnalyseActions } from './analyse-actions';
import type { BannerActionId, ChannelNumber } from './types';

type ChannelState = InstrumentState['channels'][number];

interface Deps {
  commands: InstrumentCommands;
  calibration: CalibrationCommands;
  /** Why calibration cannot open now, or null; the tag is disabled then, a shortcut or stale click must not reach the backend. */
  calibrationBlocked: () => string | null;
  /** Current values at the moment of the call; toggles need them, nothing is cached. */
  read: () => { instrument: InstrumentState | undefined; connection: ConnectionSnapshot | undefined };
  copyText: (text: string) => void;
  notify: (kind: 'error', text: string) => void;
  select: (channel: ChannelNumber) => void;
  cursors: Pick<CursorsStore, 'toggle' | 'move' | 'remove' | 'clear'>;
  canToggleCursors: () => boolean;
  pinned: { set(id: string, pinned: boolean): Promise<void> };
  files: FileActions;
  /** Hides the banner of the recording failure that is showing. */
  dismissRecordingFailure: () => void;
}

const PROBES = [1, 10, 100];

/** Accepts the typeset minus, a decimal comma and a trailing unit as the position field shows them. */
export function parseDivisions(text: string): number {
  const cleaned = text
    .trim()
    .replace('−', '-')
    .replace(',', '.')
    .replace(/\s*div$/i, '');
  return cleaned === '' ? Number.NaN : Number(cleaned);
}

type Commands = InstrumentCommands;

const range = (channel: ChannelNumber) => ({ kind: 'range', channel }) as const;

function channelActions({ commands, read, notify, select }: Deps) {
  const channelState = (channel: ChannelNumber) => read().instrument?.channels.find((c) => c.number === channel);
  const patchFrom = (channel: ChannelNumber, patch: (state: ChannelState) => ChannelPatch) => {
    const state = channelState(channel);
    if (state) commands.setChannel(channel, patch(state));
  };
  return {
    setChannelEnabled: (channel: ChannelNumber, enabled: boolean) => commands.setChannel(channel, { enabled }),
    stepVoltsPerDiv: (channel: ChannelNumber, direction: -1 | 1) => commands.step(range(channel), direction),
    resetVoltsPerDiv: (channel: ChannelNumber) => commands.reset(range(channel)),
    setChannelPosition: (channel: ChannelNumber, positionDivs: number) =>
      commands.setChannel(channel, { positionDivs }),
    commitChannelPosition: (channel: ChannelNumber, text: string) => {
      const positionDivs = parseDivisions(text);
      if (Number.isFinite(positionDivs)) commands.setChannel(channel, { positionDivs });
      else notify('error', `"${text}" is not a position in divisions`);
    },
    setCoupling: (channel: ChannelNumber, coupling: 'ac' | 'dc') => commands.setChannel(channel, { coupling }),
    toggleBandwidth: (channel: ChannelNumber) => patchFrom(channel, (s) => ({ bandwidthLimit: !s.bandwidthLimit })),
    cycleProbe: (channel: ChannelNumber) =>
      patchFrom(channel, (s) => ({ probe: PROBES[(PROBES.indexOf(s.probe) + 1) % PROBES.length] })),
    toggleInvert: (channel: ChannelNumber) => patchFrom(channel, (s) => ({ invert: !s.invert })),
    selectChannel: select,
  };
}

function horizontalAndTriggerActions(commands: Commands) {
  return {
    stepTimePerDiv: (direction: -1 | 1) => commands.step({ kind: 'timeBase' }, direction),
    resetTimePerDiv: () => commands.reset({ kind: 'timeBase' }),
    setTriggerPosition: (fraction: number) => commands.setTriggerPosition(fraction),
    stepTriggerPosition: (direction: -1 | 1) => commands.step({ kind: 'triggerPosition' }, direction),
    setTriggerLevel: (divisions: number) => commands.setTriggerLevel(divisions),
    stepTriggerLevel: (direction: -1 | 1) => commands.step({ kind: 'triggerLevel' }, direction),
    resetTriggerLevel: () => commands.reset({ kind: 'triggerLevel' }),
    setTriggerSource: (channel: ChannelNumber) => commands.setTrigger({ source: channel }),
    setTriggerSlope: (slope: 'rising' | 'falling') => commands.setTrigger({ slope }),
    setTriggerSweep: (sweep: 'auto' | 'normal') => commands.setTrigger({ sweep }),
  };
}

function bannerAction({ commands, read, copyText, dismissRecordingFailure }: Deps, id: BannerActionId) {
  switch (id) {
    case 'retry':
      return commands.retryConnection();
    case 'use-simulator':
      return commands.useSimulator(true);
    case 'choose-new-file':
      dismissRecordingFailure();
      return commands.startRecording();
    case 'dismiss':
      return dismissRecordingFailure();
    case 'copy-command': {
      const command = read().connection?.issue?.help?.installCommand;
      if (command) copyText(command);
    }
  }
}

function cursorAndMeasurementActions({ cursors, pinned, canToggleCursors }: Deps) {
  return {
    toggleCursors: () => {
      if (canToggleCursors()) cursors.toggle();
    },
    moveCursor: cursors.move,
    removeCursor: cursors.remove,
    clearCursors: cursors.clear,
    setMeasurementPinned: (id: string, on: boolean) => void pinned.set(id, on),
  };
}

function calibrationActions({ calibration, calibrationBlocked }: Deps) {
  return {
    openCalibration: () => {
      if (calibrationBlocked() === null) calibration.openCalibration();
    },
    startCalibration: calibration.startCalibration,
    calibrateChannel: calibration.calibrateChannel,
    continueChannel: calibration.continueChannel,
    closeCalibration: calibration.closeCalibration,
  };
}

function fileActions({ files, notify }: Deps) {
  const run = (action: () => Promise<void>) => {
    action().catch(() => notify('error', 'The file action failed'));
  };
  return {
    saveSetup: () => run(files.saveSetup),
    loadSetup: () => run(files.loadSetup),
    exportCsv: () => run(files.exportCsv),
    exportRaw: () => run(files.exportRaw),
    saveScreenshot: () => run(files.saveScreenshot),
  };
}

/** The analysis actions need the presentation stores; the window composes them in. */
export function createLiveActions(deps: Deps): Omit<MainWindowActions, keyof AnalyseActions> {
  const { commands, read, copyText } = deps;
  return {
    toggleRun: () => {
      const instrument = read().instrument;
      if (instrument) commands.setRunning(!instrument.running);
    },
    forceTrigger: () => commands.forceTrigger(),
    ...channelActions(deps),
    ...horizontalAndTriggerActions(commands),
    setInterpolation: commands.setInterpolation,
    ...cursorAndMeasurementActions(deps),
    ...fileActions(deps),
    ...calibrationActions(deps),
    bannerAction: (id) => bannerAction(deps, id),
    copyText,
  };
}
