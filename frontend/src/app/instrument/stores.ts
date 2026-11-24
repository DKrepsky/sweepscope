import { readonly, writable, type Readable } from 'svelte/store';
import type {
  AcquisitionStatus,
  BackendClient,
  ConnectionSnapshot,
  DisplayState,
  InstrumentFacts,
  InstrumentState,
  MeasurementsState,
  RecordingState,
  SpectrumState,
} from '../../api';

export interface InstrumentStores {
  connection: Readable<ConnectionSnapshot | undefined>;
  instrument: Readable<InstrumentState | undefined>;
  status: Readable<AcquisitionStatus | undefined>;
  facts: Readable<InstrumentFacts | undefined>;
  display: Readable<DisplayState | undefined>;
  measurements: Readable<MeasurementsState | undefined>;
  spectrum: Readable<SpectrumState | undefined>;
  recording: Readable<RecordingState | undefined>;
  /** Subscribes to the eight slice events, then loads the snapshot; events and snapshot apply by revision. */
  start(): Promise<void>;
  stop(): void;
}

interface Slice<T extends { revision: number }> {
  store: Readable<T | undefined>;
  apply(next: T | null | undefined): void;
}

/** One owner of the revision rule (ADR 0013 item 2): a value replaces the held one only when it is newer. */
function createSlice<T extends { revision: number }>(): Slice<T> {
  const held = writable<T | undefined>(undefined);
  let applied = 0;
  return {
    store: readonly(held),
    apply: (next) => {
      if (!next || next.revision <= applied) return;
      applied = next.revision;
      held.set(next);
    },
  };
}

function createSlices() {
  return {
    connection: createSlice<ConnectionSnapshot>(),
    instrument: createSlice<InstrumentState>(),
    status: createSlice<AcquisitionStatus>(),
    facts: createSlice<InstrumentFacts>(),
    display: createSlice<DisplayState>(),
    measurements: createSlice<MeasurementsState>(),
    spectrum: createSlice<SpectrumState>(),
    recording: createSlice<RecordingState>(),
  };
}

type Slices = ReturnType<typeof createSlices>;

function follow(client: BackendClient, slices: Slices): (() => void)[] {
  return [
    client.subscribe('connection_changed', slices.connection.apply),
    client.subscribe('instrument_changed', slices.instrument.apply),
    client.subscribe('status_changed', slices.status.apply),
    client.subscribe('facts_changed', slices.facts.apply),
    client.subscribe('display_changed', slices.display.apply),
    client.subscribe('measurements_changed', slices.measurements.apply),
    client.subscribe('spectrum_changed', slices.spectrum.apply),
    client.subscribe('recording_changed', slices.recording.apply),
  ];
}

function readableOf(slices: Slices) {
  return {
    connection: slices.connection.store,
    instrument: slices.instrument.store,
    status: slices.status.store,
    facts: slices.facts.store,
    display: slices.display.store,
    measurements: slices.measurements.store,
    spectrum: slices.spectrum.store,
    recording: slices.recording.store,
  };
}

export function createInstrumentStores({ client }: { client: BackendClient }): InstrumentStores {
  const slices = createSlices();
  let unsubscribe: (() => void)[] = [];
  let starting: Promise<void> | undefined;

  const load = async () => {
    unsubscribe = follow(client, slices);
    const snapshot = await client.send('get_snapshot', {});
    (Object.keys(slices) as (keyof typeof slices)[]).forEach((key) => slices[key].apply(snapshot[key] as never));
  };

  const stop = () => {
    unsubscribe.forEach((off) => off());
    unsubscribe = [];
    starting = undefined;
  };

  return {
    ...readableOf(slices),
    start: () =>
      (starting ??= load().catch((failure: unknown) => {
        stop();
        throw failure;
      })),
    stop,
  };
}
