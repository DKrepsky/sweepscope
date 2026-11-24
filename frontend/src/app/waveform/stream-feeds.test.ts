import { get } from 'svelte/store';
import { describe, expect, it } from 'vitest';
import { createFakeClient } from '../../api';
import { createRollFeed, createSpectrumFeed } from './stream-feeds';
import { buildRollItem, buildSpectrum } from './test-roll';

describe('roll feed', () => {
  it('given nobody subscribed, when read, then no stream is open', () => {
    const client = createFakeClient();
    createRollFeed(client);
    expect(client.openStreamCount('roll')).toBe(0);
  });

  it('given a subscriber, when it leaves, then the stream is closed and the view forgotten', () => {
    const client = createFakeClient();
    const feed = createRollFeed(client);
    const stop = feed.subscribe(() => {});
    expect(client.openStreamCount('roll')).toBe(1);
    client.pushStream('roll', buildRollItem({ samples: [1, 2] }));
    stop();
    expect(client.openStreamCount('roll')).toBe(0);
    expect(get(feed)).toBeUndefined();
  });

  it('given roll items, when they arrive, then the view grows with each', () => {
    const client = createFakeClient();
    const feed = createRollFeed(client);
    const seen: number[] = [];
    feed.subscribe((view) => view && seen.push(view.channels[0]?.samples.filter((v) => !Number.isNaN(v)).length ?? 0));
    client.pushStream('roll', buildRollItem({ sequence: 1, firstIndex: 0, samples: [1, 2] }));
    client.pushStream('roll', buildRollItem({ sequence: 2, firstIndex: 2, samples: [3] }));
    expect(seen).toEqual([2, 3]);
  });

  it('given a missing sequence number, when the next item arrives, then the skipped count grows and the gap shows', () => {
    const client = createFakeClient();
    const feed = createRollFeed(client);
    feed.subscribe(() => {});
    client.pushStream('roll', buildRollItem({ sequence: 1, firstIndex: 0, samples: [1, 2] }));
    client.pushStream('roll', buildRollItem({ sequence: 4, firstIndex: 8, samples: [3] }));
    expect(feed.stats().skipped).toBe(2);
    expect(get(feed)?.gapCount).toBe(1);
  });

  it('given a message that does not decode, when it arrives, then it is counted and the view is kept', () => {
    const client = createFakeClient();
    const feed = createRollFeed(client);
    feed.subscribe(() => {});
    client.pushStream('roll', buildRollItem({ sequence: 1 }));
    client.pushStream('roll', new ArrayBuffer(3));
    expect(feed.stats()).toMatchObject({ received: 1, rejected: 1 });
    expect(get(feed)).toBeDefined();
  });

  it('given a scripted stream, when subscribed, then the scripted items build the view', () => {
    const client = createFakeClient();
    client.scriptStream('roll', [
      buildRollItem({ sequence: 1, samples: [1] }),
      buildRollItem({ sequence: 2, firstIndex: 1, samples: [2] }),
    ]);
    const feed = createRollFeed(client);
    feed.subscribe(() => {});
    expect(get(feed)?.channels[0]?.samples.filter((v) => !Number.isNaN(v))).toHaveLength(2);
  });

  it('given a subscriber that left and another that came, when an item arrives, then the new window holds only that item', () => {
    const client = createFakeClient();
    const feed = createRollFeed(client);
    feed.subscribe(() => {})();
    feed.subscribe(() => {});
    client.pushStream('roll', buildRollItem({ sequence: 1, firstIndex: 50, samples: [1] }));
    expect(get(feed)?.channels[0]?.samples.filter((v) => !Number.isNaN(v))).toHaveLength(1);
  });
});

describe('spectrum feed', () => {
  it('given a subscriber, when a spectrum arrives, then it is published', () => {
    const client = createFakeClient();
    const feed = createSpectrumFeed(client);
    feed.subscribe(() => {});
    client.pushStream('spectrum', buildSpectrum({ sequence: 3 }));
    expect(get(feed)?.sequence).toBe(3);
  });

  it('given two spectra, when both arrive, then the newest is held', () => {
    const client = createFakeClient();
    const feed = createSpectrumFeed(client);
    feed.subscribe(() => {});
    client.pushStream('spectrum', buildSpectrum({ sequence: 3 }));
    client.pushStream('spectrum', buildSpectrum({ sequence: 4 }));
    expect(get(feed)?.sequence).toBe(4);
  });

  it('given the last subscriber leaves, when read, then the stream is closed and the spectrum forgotten', () => {
    const client = createFakeClient();
    const feed = createSpectrumFeed(client);
    const stop = feed.subscribe(() => {});
    client.pushStream('spectrum', buildSpectrum());
    stop();
    expect(client.openStreamCount('spectrum')).toBe(0);
    expect(get(feed)).toBeUndefined();
  });

  it('given a corrupt spectrum, when it arrives, then it is counted and nothing is published', () => {
    const client = createFakeClient();
    const feed = createSpectrumFeed(client);
    feed.subscribe(() => {});
    client.pushStream('spectrum', new ArrayBuffer(8));
    expect(feed.stats().rejected).toBe(1);
    expect(get(feed)).toBeUndefined();
  });
});
