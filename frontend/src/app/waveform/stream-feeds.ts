import { readable, type Readable } from 'svelte/store';
import type { BackendClient, StreamKind } from '../../api';
import { decodeRollItem, type RollItem } from './decode-roll';
import { decodeSpectrum, type DecodedSpectrum } from './decode-spectrum';
import { createRollWindow, type RollView } from './roll-window';

export interface FeedStats {
  received: number;
  /** Messages that did not decode. */
  rejected: number;
  /** Sequence numbers that never arrived: the backend or the transport dropped those messages. */
  skipped: number;
}

export type Feed<V> = Readable<V | undefined> & { stats(): FeedStats };

interface Source<M extends { sequence: number }, V> {
  kind: StreamKind;
  decode(data: ArrayBuffer): M;
  /** Called once per subscription, so state such as a roll window never outlives its stream. */
  startPublishing(): (message: M) => V;
}

/** The stream is open only while somebody subscribes; the value is forgotten when the last one leaves. */
function createStreamFeed<M extends { sequence: number }, V>(client: BackendClient, source: Source<M, V>): Feed<V> {
  const counts: FeedStats = { received: 0, rejected: 0, skipped: 0 };
  const store = readable<V | undefined>(undefined, (set) => {
    const publish = source.startPublishing();
    let lastSequence: number | undefined;
    const handle = client.openStream(source.kind, (data) => {
      let message: M;
      try {
        message = source.decode(data);
      } catch {
        counts.rejected++;
        return;
      }
      counts.received++;
      if (lastSequence !== undefined && message.sequence > lastSequence + 1) {
        counts.skipped += message.sequence - lastSequence - 1;
      }
      lastSequence = message.sequence;
      set(publish(message));
    });
    return () => {
      handle.close();
      set(undefined);
    };
  });
  return { subscribe: store.subscribe, stats: () => ({ ...counts }) };
}

/** Owns the roll stream: every item goes into one window and its view is published after each. */
export function createRollFeed(client: BackendClient): Feed<RollView> {
  return createStreamFeed<RollItem, RollView>(client, {
    kind: 'roll',
    decode: decodeRollItem,
    startPublishing: () => {
      const window = createRollWindow();
      return (item) => {
        window.push(item);
        return window.view();
      };
    },
  });
}

export function createSpectrumFeed(client: BackendClient): Feed<DecodedSpectrum> {
  return createStreamFeed<DecodedSpectrum, DecodedSpectrum>(client, {
    kind: 'spectrum',
    decode: decodeSpectrum,
    startPublishing: () => (spectrum) => spectrum,
  });
}
