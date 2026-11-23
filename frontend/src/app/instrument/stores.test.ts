import { get } from 'svelte/store';
import { describe, expect, it } from 'vitest';
import { createFakeClient } from '../../api';
import { createInstrumentStores } from './stores';
import { acquisitionStatus, connectionSnapshot, instrumentFacts, instrumentState, stateSnapshot } from './test-data';

function setup(snapshot = stateSnapshot()) {
  const client = createFakeClient();
  client.respond('get_snapshot', () => snapshot);
  return { client, stores: createInstrumentStores({ client }) };
}

describe('instrument stores', () => {
  it('given stores that have not started, when read, then every slice is undefined', () => {
    const { stores } = setup();
    const slices = [get(stores.connection), get(stores.instrument), get(stores.status), get(stores.facts)];
    expect(slices).toEqual([undefined, undefined, undefined, undefined]);
  });

  it('given a snapshot, when started, then every slice holds the snapshot value', async () => {
    const snapshot = stateSnapshot();
    const { stores } = setup(snapshot);
    await stores.start();
    expect(get(stores.connection)).toEqual(snapshot.connection);
    expect(get(stores.instrument)).toEqual(snapshot.instrument);
    expect(get(stores.status)).toEqual(snapshot.status);
    expect(get(stores.facts)).toEqual(snapshot.facts);
  });

  it('given a snapshot without an instrument, when started, then the instrument store stays undefined', async () => {
    const { stores } = setup(stateSnapshot({ instrument: null }));
    await stores.start();
    expect(get(stores.instrument)).toBeUndefined();
  });

  it('given start, when the snapshot is requested, then the four events are already subscribed', async () => {
    const { client, stores } = setup();
    let early: unknown;
    client.respond('get_snapshot', () => {
      client.emit('status_changed', acquisitionStatus({ revision: 5, phase: 'auto' }));
      early = get(stores.status);
      return stateSnapshot();
    });
    await stores.start();
    expect(early).toEqual(acquisitionStatus({ revision: 5, phase: 'auto' }));
  });

  it('given a snapshot that arrives after a newer event, when applied, then the event wins', async () => {
    const { client, stores } = setup();
    client.respond('get_snapshot', () => {
      client.emit('instrument_changed', instrumentState({ revision: 7, running: false }));
      return stateSnapshot({ instrument: instrumentState({ revision: 3, running: true }) });
    });
    await stores.start();
    expect(get(stores.instrument)?.revision).toBe(7);
    expect(get(stores.instrument)?.running).toBe(false);
  });

  it('given an applied revision, when an event with a lower revision arrives, then it is ignored', async () => {
    const { client, stores } = setup();
    await stores.start();
    client.emit('connection_changed', connectionSnapshot({ revision: 4, state: 'disconnected' }));
    client.emit('connection_changed', connectionSnapshot({ revision: 3, state: 'ready' }));
    expect(get(stores.connection)?.state).toBe('disconnected');
  });

  it('given an applied revision, when an event with the same revision arrives, then it is ignored', async () => {
    const { client, stores } = setup();
    await stores.start();
    client.emit('facts_changed', instrumentFacts({ revision: 1, deadTime: { value: 1, text: '1', unit: 's' } }));
    expect(get(stores.facts)?.deadTime).toBeNull();
  });

  it('given an applied revision, when a newer event arrives, then the store follows it', async () => {
    const { client, stores } = setup();
    await stores.start();
    client.emit('status_changed', acquisitionStatus({ revision: 2, phase: 'triggered' }));
    expect(get(stores.status)?.phase).toBe('triggered');
  });

  it('given stores that have not started, when an event arrives, then nothing is stored', () => {
    const { client, stores } = setup();
    client.emit('status_changed', acquisitionStatus({ revision: 2 }));
    expect(get(stores.status)).toBeUndefined();
  });

  it('given a failing snapshot, when started, then start rejects and the stores stay unchanged', async () => {
    const { client, stores } = setup();
    client.respond('get_snapshot', () => Promise.reject({ category: 'fatal', message: 'boom' }));
    await expect(stores.start()).rejects.toEqual({ category: 'fatal', message: 'boom' });
    expect(get(stores.connection)).toBeUndefined();
  });

  it('given started stores, when stopped, then events no longer change them', async () => {
    const { client, stores } = setup();
    await stores.start();
    stores.stop();
    client.emit('status_changed', acquisitionStatus({ revision: 9, phase: 'auto' }));
    expect(get(stores.status)?.revision).toBe(1);
  });

  it('given started stores, when started again, then the snapshot is requested once', async () => {
    const { client, stores } = setup();
    await stores.start();
    await stores.start();
    expect(client.calls.filter((call) => call.name === 'get_snapshot')).toHaveLength(1);
  });
});
