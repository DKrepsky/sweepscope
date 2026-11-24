import { derived, get, writable, type Readable } from 'svelte/store';
import { selectCalibrationDialog, selectCalibrationTag, type CalibrationCommands } from '../calibration';
import type { CursorsStore } from '../cursors';
import type { FileActions } from '../files';
import type { InstrumentCommands, InstrumentStores } from '../instrument';
import {
  createPeakMarkerStore,
  createPersistenceEpoch,
  createViewStore,
  createXyAxesStore,
  type AnalyseCommands,
  type AnalysisSources,
} from '../views';
import type { DecodedFrame } from '../waveform';
import type { MainWindowActions } from './actions';
import { createAnalyseActions } from './analyse-actions';
import { createFailureDismissal } from '../recording';
import { calibrationBlockedReason } from '../dialog-guard';
import { createLiveActions } from './live-actions';
import { selectBanners } from './select-banners';
import type { PinnedMeasurements } from './pinned-measurements';
import { selectAutoset } from './select-autoset';
import { selectChannels } from './select-channels';
import { selectCursors } from './select-cursors';
import { selectGraph } from './select-graph';
import { selectHorizontal } from './select-horizontal';
import { selectMath } from './select-math';
import { selectMeasurements } from './select-measurements';
import { selectPeak } from './select-peak';
import { selectRecording } from './select-recording';
import { selectRoll } from './select-roll';
import { selectStatus } from './select-status';
import { selectToolbar } from './select-toolbar';
import { selectTrigger } from './select-trigger';
import { selectPersistence, selectViews } from './select-views';
import { channelNumber, type ModelInputs } from './select-common';
import type { ChannelNumber, MainWindowModel } from './types';

export interface LiveMainWindow extends Readable<MainWindowModel>, MainWindowActions {}

interface Deps {
  stores: Pick<
    InstrumentStores,
    'connection' | 'instrument' | 'status' | 'facts' | 'display' | 'measurements' | 'zeroCalibration'
  >;
  analysis: AnalysisSources;
  commands: InstrumentCommands;
  calibration: CalibrationCommands;
  analyseCommands: AnalyseCommands;
  cursors: CursorsStore;
  pinned: PinnedMeasurements;
  files: FileActions;
  frames: Readable<DecodedFrame | undefined>;
  copyText: (text: string) => void;
  notify: (kind: 'error', text: string) => void;
  renderer: string;
}

const selectCalibration = (i: ModelInputs): MainWindowModel['calibration'] => ({
  tag: selectCalibrationTag(i.zeroCalibration, i.connection, i.recording, i.instrument),
  dialog: selectCalibrationDialog(i.zeroCalibration),
});

function selectModel(i: ModelInputs): MainWindowModel {
  const { connection, instrument, selected } = i;
  const views = selectViews(i);
  const persistence = selectPersistence(i);
  const roll = selectRoll(i);
  const graph = selectGraph(i, { views, persistence, roll });
  return {
    toolbar: selectToolbar(i, {
      hasWaveform: graph.state !== 'empty',
      cursorsOn: Object.keys(i.cursors).length > 0,
    }),
    horizontal: selectHorizontal(
      connection,
      instrument,
      i.facts,
      i.display?.interpolation ?? 'linear',
      i.display?.automatic ?? false,
    ),
    channels: selectChannels(connection, instrument, selected, i.measurements, i.view === 'xy' ? i.xyAxes : undefined),
    trigger: selectTrigger(connection, instrument, i.pulseWidthRejected),
    cursors: selectCursors(i.cursors, instrument, graph.state, selected),
    measurements: selectMeasurements(i.measurements, i.pinned, graph.state, {
      roll: instrument?.horizontal.roll ?? false,
      ...(instrument && { mathUnit: instrument.math.unit }),
    }),
    status: selectStatus(connection, instrument, i.facts, i.renderer, i.measurements),
    graph,
    views,
    persistence,
    banners: selectBanners(i),
    peak: selectPeak(i),
    roll,
    recording: selectRecording(i),
    autoset: selectAutoset(i),
    math: selectMath(i),
    calibration: selectCalibration(i),
  };
}

interface Presentation {
  selected: Readable<ChannelNumber>;
  view: Readable<ModelInputs['view']>;
  xyAxes: Readable<ModelInputs['xyAxes']>;
  peak: Readable<ModelInputs['peak']>;
  persistenceEpoch: Readable<number>;
  pulseWidthRejected: Readable<boolean>;
  dismissedRecordingFailure: Readable<number>;
}

type BackendInputs = Pick<
  ModelInputs,
  'connection' | 'instrument' | 'status' | 'facts' | 'display' | 'measurements' | 'zeroCalibration' | 'frame'
>;
type AnalysisInputs = Pick<ModelInputs, 'spectrum' | 'spectrumFrame' | 'recording' | 'roll'>;
type PresentationInputs = Pick<
  ModelInputs,
  'selected' | 'view' | 'xyAxes' | 'peak' | 'persistenceEpoch' | 'dismissedRecordingFailure' | 'pulseWidthRejected'
>;

function backendInputs({ stores, frames }: Deps): Readable<BackendInputs> {
  const { connection, instrument, status, facts, display, measurements, zeroCalibration } = stores;
  return derived(
    [connection, instrument, status, facts, display, measurements, zeroCalibration, frames],
    ([connection, instrument, status, facts, display, measurements, zeroCalibration, frame]) => ({
      connection,
      instrument,
      status,
      facts,
      display,
      measurements,
      zeroCalibration,
      frame,
    }),
  );
}

function analysisInputs({ analysis }: Deps): Readable<AnalysisInputs> {
  return derived(
    [analysis.spectrum, analysis.spectrumFrame, analysis.recording, analysis.roll],
    ([spectrum, spectrumFrame, recording, roll]) => ({ spectrum, spectrumFrame, recording, roll }),
  );
}

function presentationInputs(p: Presentation): Readable<PresentationInputs> {
  return derived(
    [p.selected, p.view, p.xyAxes, p.peak, p.persistenceEpoch, p.dismissedRecordingFailure, p.pulseWidthRejected],
    ([selected, view, xyAxes, peak, persistenceEpoch, dismissedRecordingFailure, pulseWidthRejected]) => ({
      selected,
      view,
      xyAxes,
      peak,
      persistenceEpoch,
      pulseWidthRejected,
      dismissedRecordingFailure,
    }),
  );
}

function deriveModel(deps: Deps, presentation: Presentation): Readable<MainWindowModel> {
  const { cursors, pinned, renderer } = deps;
  return derived(
    [backendInputs(deps), analysisInputs(deps), presentationInputs(presentation), cursors, pinned],
    ([backend, analysed, shown, cursorState, pinnedIds]) =>
      selectModel({ ...backend, ...analysed, ...shown, cursors: cursorState, pinned: pinnedIds, renderer }),
  );
}

const enabledChannels = (instrument: ModelInputs['instrument']): ChannelNumber[] =>
  (instrument?.channels ?? []).filter((c) => c.enabled).map((c) => channelNumber(c.number));

function presentation(analyseCommands: AnalyseCommands) {
  return {
    selected: writable<ChannelNumber>(1),
    view: createViewStore({ setSpectrumActive: (active) => analyseCommands.setSpectrum({ active }) }),
    xyAxes: createXyAxesStore(),
    peak: createPeakMarkerStore(),
    persistenceEpoch: createPersistenceEpoch(),
    dismissedRecordingFailure: createFailureDismissal(),
    pulseWidthRejected: writable(false),
  };
}

/** The key and the toast's Retry press this too: a request while it cannot run is dropped instead of reaching the backend. */
const whenAvailable = (model: Readable<MainWindowModel>, run: () => void) => () => {
  if (get(model).autoset.available.enabled) run();
};

/**
 * Colour and label are fixed by tokens; channel selection, cursors, view, XY axes and the peak marker are presentation
 * state, never sent to the backend.
 */
export function createLiveMainWindow(deps: Deps): LiveMainWindow {
  const { stores, analysis, commands, calibration, analyseCommands, cursors, pinned, files, copyText, notify } = deps;
  const shown = presentation(analyseCommands);
  const { selected, view, xyAxes: xy, peak, persistenceEpoch, dismissedRecordingFailure: failureDismissal } = shown;
  const { pulseWidthRejected } = shown;
  const model = deriveModel(deps, shown);

  const actions = createLiveActions({
    commands,
    calibration,
    calibrationBlocked: () => calibrationBlockedReason(get(model).calibration.tag),
    read: () => ({ instrument: get(stores.instrument), connection: get(stores.connection) }),
    copyText,
    notify,
    select: (channel) => {
      selected.set(channel);
      peak.follow(channel);
    },
    cursors,
    canToggleCursors: () => get(model).toolbar.cursorsAvailability.enabled,
    pinned,
    files,
    dismissRecordingFailure: () => failureDismissal.dismiss(get(analysis.recording)?.failure?.id ?? 0),
  });
  const analyseActions = createAnalyseActions({
    commands: analyseCommands,
    instrument: commands,
    view,
    xy,
    peak,
    persistenceEpoch,
    pulseWidthRejected,
    enabledChannels: () => enabledChannels(get(stores.instrument)),
    binCount: () => get(analysis.spectrumFrame)?.bins ?? 0,
  });

  const startAutoset = whenAvailable(model, analyseActions.startAutoset);
  return { subscribe: model.subscribe, ...actions, ...analyseActions, startAutoset };
}
