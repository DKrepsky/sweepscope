import { get } from 'svelte/store';
import { describe, expect, it } from 'vitest';
import { createFakeClient } from '../../api';
import { createFrameFeed } from './frame-feed';
import { buildFrame } from './test-frame';

function setup(nowMs = 0) {
  const client = createFakeClient();
  const pending: (() => void)[] = [];
  const clock = { nowMs };
  const feed = createFrameFeed({
    client,
    requestFrame: (callback) => void pending.push(callback),
    now: () => clock.nowMs,
  });
  const tick = () => pending.splice(0).forEach((callback) => callback());
  return { client, feed, tick, pending, clock };
}

describe('frame feed', () => {
  it('given a feed that has not started, when read, then it holds no frame and no stream is open', () => {
    const { client, feed } = setup();
    expect(get(feed.frame)).toBeUndefined();
    expect(client.openStreamCount('frames')).toBe(0);
  });

  it('given a started feed, when a frame arrives, then it is published at the next animation frame and not before', () => {
    const { client, feed, tick } = setup();
    feed.start();
    client.pushStream('frames', buildFrame({ sequence: 5 }));
    expect(get(feed.frame)).toBeUndefined();
    tick();
    expect(get(feed.frame)?.sequence).toBe(5);
  });

  it('given three frames within one animation frame, when it fires, then only the newest is published', () => {
    const { client, feed, tick } = setup();
    const seen: (number | undefined)[] = [];
    feed.frame.subscribe((frame) => seen.push(frame?.sequence));
    feed.start();
    [1, 2, 3].forEach((sequence) => client.pushStream('frames', buildFrame({ sequence })));
    tick();
    expect(seen).toEqual([undefined, 3]);
  });

  it('given three frames within one animation frame, when they arrive, then one animation frame is requested', () => {
    const { client, feed, pending } = setup();
    feed.start();
    [1, 2, 3].forEach((sequence) => client.pushStream('frames', buildFrame({ sequence })));
    expect(pending).toHaveLength(1);
  });

  it('given no new frame, when animation frames fire, then the held frame is not published again', () => {
    const { client, feed, tick } = setup();
    let publishes = 0;
    feed.frame.subscribe(() => publishes++);
    feed.start();
    client.pushStream('frames', buildFrame());
    tick();
    tick();
    tick();
    expect(publishes).toBe(2);
  });

  it('given a frame that does not decode, when it arrives, then the held frame stays and it is counted as rejected', () => {
    const { client, feed, tick } = setup();
    feed.start();
    client.pushStream('frames', buildFrame({ sequence: 1 }));
    tick();
    client.pushStream('frames', new ArrayBuffer(3));
    tick();
    expect(get(feed.frame)?.sequence).toBe(1);
    expect(feed.stats()).toMatchObject({ received: 1, rejected: 1 });
  });

  it('given frames replaced before display, when stats are read, then they count received, published and replaced', () => {
    const { client, feed, tick } = setup();
    feed.start();
    [1, 2, 3].forEach((sequence) => client.pushStream('frames', buildFrame({ sequence })));
    tick();
    expect(feed.stats()).toMatchObject({ received: 3, published: 1, replaced: 2 });
  });

  it('given a frame sent at a known time, when published, then the latency is the clock minus the sent time', () => {
    const { client, feed, tick, clock } = setup();
    feed.start();
    client.pushStream('frames', buildFrame({ sentUs: 1_000_000 }));
    clock.nowMs = 1_000 + 12;
    tick();
    expect(feed.stats().latencyMs).toBe(12);
  });

  it('given a started feed, when stopped, then the stream closes, later frames are ignored and the last frame stays', () => {
    const { client, feed, tick } = setup();
    feed.start();
    client.pushStream('frames', buildFrame({ sequence: 1 }));
    tick();
    feed.stop();
    client.pushStream('frames', buildFrame({ sequence: 2 }));
    tick();
    expect(client.openStreamCount('frames')).toBe(0);
    expect(get(feed.frame)?.sequence).toBe(1);
  });

  it('given a frame waiting for its animation frame, when the feed stops, then it is not published', () => {
    const { client, feed, tick } = setup();
    feed.start();
    client.pushStream('frames', buildFrame({ sequence: 9 }));
    feed.stop();
    tick();
    expect(get(feed.frame)).toBeUndefined();
  });

  it('given a started feed, when started again, then only one stream is open', () => {
    const { client, feed } = setup();
    feed.start();
    feed.start();
    expect(client.openStreamCount('frames')).toBe(1);
  });
});
