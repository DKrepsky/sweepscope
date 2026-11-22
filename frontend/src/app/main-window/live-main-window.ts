import { derived, get, writable, type Readable } from 'svelte/store';
import type { InstrumentCommands, InstrumentStores } from '../instrument';
import type { DecodedFrame } from '../waveform';
import type { MainWindowActions } from './actions';
import { createLiveActions } from './live-actions';
import { mockMeasurements } from './mock-model';
import { selectBanners } from './select-banners';
import { selectChannels } from './select-channels';
import { selectGraph } from './select-graph';
import { selectHorizontal } from './select-horizontal';
import { selectStatus } from './select-status';
import { selectToolbar } from './select-toolbar';
import { selectTrigger } from './select-trigger';
import type { ChannelNumber, CursorsModel, Interpolation, MainWindowModel } from './types';

export interface LiveMainWindow extends Readable<MainWindowModel>, MainWindowActions {}

interface Deps {
  stores: Pick<InstrumentStores, 'connection' | 'instrument' | 'status' | 'facts'>;
  commands: InstrumentCommands;
  frames: Readable<DecodedFrame | undefined>;
  copyText: (text: string) => void;
  notify: (kind: 'error', text: string) => void;
  renderer: string;
}

/** MOCK: cursors and measurements arrive with their own stories; the sidebar shows the placeholder data meanwhile. */
const cursors = (): CursorsModel => ({
  enabled: false,
  source: 1,
  values: [],
  originNote: 'Cursors arrive in a later release.',
  lines: { x: [], y: [] },
  tags: [],
});

/** Colour and label are fixed by tokens; selection and interpolation are presentation state, never sent to the backend. */
export function createLiveMainWindow({ stores, commands, frames, copyText, notify, renderer }: Deps): LiveMainWindow {
  const selected = writable<ChannelNumber>(1);
  const interpolation = writable<Interpolation>('linear');

  const model = derived(
    [stores.connection, stores.instrument, stores.status, stores.facts, frames, selected, interpolation],
    ([connection, instrument, status, facts, frame, channel, interpolate]): MainWindowModel => ({
      toolbar: selectToolbar(connection, instrument, status),
      horizontal: selectHorizontal(connection, instrument, facts, interpolate),
      channels: selectChannels(connection, instrument, channel),
      trigger: selectTrigger(connection, instrument),
      cursors: cursors(),
      measurements: mockMeasurements(),
      status: selectStatus(connection, instrument, facts, renderer),
      graph: selectGraph(connection, instrument, frame, channel),
      banners: selectBanners(connection),
    }),
  );

  const actions = createLiveActions({
    commands,
    read: () => ({ instrument: get(stores.instrument), connection: get(stores.connection) }),
    copyText,
    notify,
    select: (channel) => selected.set(channel),
    interpolate: (choice) => interpolation.set(choice),
  });

  return { subscribe: model.subscribe, ...actions };
}
