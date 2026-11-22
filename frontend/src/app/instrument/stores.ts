import { readonly, writable, type Readable } from 'svelte/store';
import type { AcquisitionStatus, BackendClient, ConnectionSnapshot, InstrumentFacts, InstrumentState } from '../../api';

export interface InstrumentStores {
  connection: Readable<ConnectionSnapshot | undefined>;
  instrument: Readable<InstrumentState | undefined>;
  status: Readable<AcquisitionStatus | undefined>;
  facts: Readable<InstrumentFacts | undefined>;
  /** Subscribes to the four slice events, then loads the snapshot; events and snapshot apply by revision. */
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

export function createInstrumentStores({ client }: { client: BackendClient }): InstrumentStores {
  const connection = createSlice<ConnectionSnapshot>();
  const instrument = createSlice<InstrumentState>();
  const status = createSlice<AcquisitionStatus>();
  const facts = createSlice<InstrumentFacts>();
  let unsubscribe: (() => void)[] = [];
  let starting: Promise<void> | undefined;

  const follow = () => [
    client.subscribe('connection_changed', connection.apply),
    client.subscribe('instrument_changed', instrument.apply),
    client.subscribe('status_changed', status.apply),
    client.subscribe('facts_changed', facts.apply),
  ];

  const load = async () => {
    unsubscribe = follow();
    const snapshot = await client.send('get_snapshot', {});
    connection.apply(snapshot.connection);
    instrument.apply(snapshot.instrument);
    status.apply(snapshot.status);
    facts.apply(snapshot.facts);
  };

  const stop = () => {
    unsubscribe.forEach((off) => off());
    unsubscribe = [];
    starting = undefined;
  };

  return {
    connection: connection.store,
    instrument: instrument.store,
    status: status.store,
    facts: facts.store,
    start: () =>
      (starting ??= load().catch((failure: unknown) => {
        stop();
        throw failure;
      })),
    stop,
  };
}
