import { derived, get, writable, type Readable } from 'svelte/store';
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
  stores: Pick<InstrumentStores, 'connection' | 'instrument' | 'status' | 'facts' | 'display' | 'measurements'>;
  analysis: AnalysisSources;
  commands: InstrumentCommands;
  analyseCommands: AnalyseCommands;
  cursors: CursorsStore;
  pinned: PinnedMeasurements;
  files: FileActions;
  frames: Readable<DecodedFrame | undefined>;
  copyText: (text: string) => void;
  notify: (kind: 'error', text: string) => void;
  renderer: string;
}

function selectModel(i: ModelInputs): MainWindowModel {
  const { connection, instrument, selected } = i;
  const graph = selectGraph(connection, instrument, i.frame, selected);
  return {
    toolbar: selectToolbar(connection, instrument, i.status, {
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
    channels: selectChannels(connection, instrument, selected, i.measurements),
    trigger: selectTrigger(connection, instrument),
    cursors: selectCursors(i.cursors, instrument, graph.state, selected),
    measurements: selectMeasurements(i.measurements, i.pinned, graph.state),
    status: selectStatus(connection, instrument, i.facts, i.renderer, i.measurements),
    graph,
    banners: selectBanners(connection),
    views: selectViews(i),
    persistence: selectPersistence(i),
    peak: selectPeak(i),
    roll: selectRoll(i),
    recording: selectRecording(i),
    autoset: selectAutoset(i),
    math: selectMath(i),
  };
}

interface Presentation {
  selected: Readable<ChannelNumber>;
  view: Readable<ModelInputs['view']>;
  xyAxes: Readable<ModelInputs['xyAxes']>;
  peak: Readable<ModelInputs['peak']>;
  persistenceEpoch: Readable<number>;
}

type BackendInputs = Pick<
  ModelInputs,
  'connection' | 'instrument' | 'status' | 'facts' | 'display' | 'measurements' | 'frame'
>;
type AnalysisInputs = Pick<ModelInputs, 'spectrum' | 'spectrumFrame' | 'recording' | 'roll'>;
type PresentationInputs = Pick<ModelInputs, 'selected' | 'view' | 'xyAxes' | 'peak' | 'persistenceEpoch'>;

function backendInputs({ stores, frames }: Deps): Readable<BackendInputs> {
  const { connection, instrument, status, facts, display, measurements } = stores;
  return derived(
    [connection, instrument, status, facts, display, measurements, frames],
    ([connection, instrument, status, facts, display, measurements, frame]) => ({
      connection,
      instrument,
      status,
      facts,
      display,
      measurements,
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
    [p.selected, p.view, p.xyAxes, p.peak, p.persistenceEpoch],
    ([selected, view, xyAxes, peak, persistenceEpoch]) => ({ selected, view, xyAxes, peak, persistenceEpoch }),
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

/**
 * Colour and label are fixed by tokens; channel selection, cursors, view, XY axes and the peak marker are presentation
 * state, never sent to the backend.
 */
export function createLiveMainWindow(deps: Deps): LiveMainWindow {
  const { stores, analysis, commands, analyseCommands, cursors, pinned, files, copyText, notify } = deps;
  const selected = writable<ChannelNumber>(1);
  const view = createViewStore({ setSpectrumActive: (active) => analyseCommands.setSpectrum({ active }) });
  const xy = createXyAxesStore();
  const peak = createPeakMarkerStore();
  const persistenceEpoch = createPersistenceEpoch();

  const model = deriveModel(deps, { selected, view, xyAxes: xy, peak, persistenceEpoch });

  const actions = createLiveActions({
    commands,
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
  });
  const analyseActions = createAnalyseActions({
    commands: analyseCommands,
    instrument: commands,
    view,
    xy,
    peak,
    persistenceEpoch,
    enabledChannels: () => enabledChannels(get(stores.instrument)),
    binCount: () => get(analysis.spectrumFrame)?.bins ?? 0,
  });

  return { subscribe: model.subscribe, ...actions, ...analyseActions };
}
