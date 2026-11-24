import { derived, get, writable, type Readable } from 'svelte/store';
import type {
  AcquisitionStatus,
  ConnectionSnapshot,
  DisplayState,
  InstrumentFacts,
  InstrumentState,
  MeasurementsState,
} from '../../api';
import type { CursorsState, CursorsStore } from '../cursors';
import type { FileActions } from '../files';
import type { InstrumentCommands, InstrumentStores } from '../instrument';
import type { DecodedFrame } from '../waveform';
import type { MainWindowActions } from './actions';
import { createLiveActions } from './live-actions';
import { selectBanners } from './select-banners';
import type { PinnedMeasurements } from './pinned-measurements';
import { selectChannels } from './select-channels';
import { selectCursors } from './select-cursors';
import { selectGraph } from './select-graph';
import { selectHorizontal } from './select-horizontal';
import { selectMeasurements } from './select-measurements';
import { selectStatus } from './select-status';
import { selectToolbar } from './select-toolbar';
import { selectTrigger } from './select-trigger';
import type { ChannelNumber, MainWindowModel } from './types';

export interface LiveMainWindow extends Readable<MainWindowModel>, MainWindowActions {}

interface Deps {
  stores: Pick<InstrumentStores, 'connection' | 'instrument' | 'status' | 'facts' | 'display' | 'measurements'>;
  commands: InstrumentCommands;
  cursors: CursorsStore;
  pinned: PinnedMeasurements;
  files: FileActions;
  frames: Readable<DecodedFrame | undefined>;
  copyText: (text: string) => void;
  notify: (kind: 'error', text: string) => void;
  renderer: string;
}

interface Inputs {
  connection: ConnectionSnapshot | undefined;
  instrument: InstrumentState | undefined;
  status: AcquisitionStatus | undefined;
  facts: InstrumentFacts | undefined;
  display: DisplayState | undefined;
  measurements: MeasurementsState | undefined;
  frame: DecodedFrame | undefined;
  selected: ChannelNumber;
  cursors: CursorsState;
  pinned: readonly string[];
  renderer: string;
}

function selectModel(i: Inputs): MainWindowModel {
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
    status: selectStatus(connection, instrument, i.facts, i.renderer),
    graph,
    banners: selectBanners(connection),
  };
}

function deriveModel(
  { stores, cursors, pinned, frames, renderer }: Deps,
  selected: Readable<ChannelNumber>,
): Readable<MainWindowModel> {
  const { connection, instrument, status, facts, display, measurements } = stores;
  return derived(
    [connection, instrument, status, facts, display, measurements, frames, selected, cursors, pinned],
    ([connection, instrument, status, facts, display, measurements, frame, channel, cursorState, pinnedIds]) =>
      selectModel({
        connection,
        instrument,
        status,
        facts,
        display,
        measurements,
        frame,
        selected: channel,
        cursors: cursorState,
        pinned: pinnedIds,
        renderer,
      }),
  );
}

/** Colour and label are fixed by tokens; channel selection and cursors are presentation state, never sent to the backend. */
export function createLiveMainWindow(deps: Deps): LiveMainWindow {
  const { stores, commands, cursors, pinned, files, copyText, notify } = deps;
  const selected = writable<ChannelNumber>(1);

  const model = deriveModel(deps, selected);

  const actions = createLiveActions({
    commands,
    read: () => ({ instrument: get(stores.instrument), connection: get(stores.connection) }),
    copyText,
    notify,
    select: (channel) => selected.set(channel),
    cursors,
    canToggleCursors: () => get(model).toolbar.cursorsAvailability.enabled,
    pinned,
    files,
  });

  return { subscribe: model.subscribe, ...actions };
}
