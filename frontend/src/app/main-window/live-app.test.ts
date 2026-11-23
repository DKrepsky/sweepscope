import { get } from 'svelte/store';
import { describe, expect, it, vi } from 'vitest';
import { createFakeClient } from '../../api';
import { stateSnapshot } from '../instrument/test-data';
import type { FrameBudgetProbe } from '../waveform';
import { buildFrame } from '../waveform/test-frame';
import { createLiveApp } from './live-app';

function setup() {
  const client = createFakeClient();
  client.respond('get_snapshot', () => stateSnapshot());
  const animationFrames: (() => void)[] = [];
  const order: string[] = [];
  const probe: FrameBudgetProbe = {
    beginFrame: () => order.push('begin'),
    endFrame: () => order.push('end'),
  };
  const drawn = Promise.resolve();
  const app = createLiveApp({
    client,
    notify: vi.fn(),
    copyText: vi.fn(),
    composeScreenshot: () => Promise.resolve(new Uint8Array()),
    now: () => 0,
    requestAnimationFrame: (callback) => void animationFrames.push(callback),
    afterRender: () => drawn,
    probe,
  });
  const runAnimationFrame = () => animationFrames.splice(0).forEach((callback) => callback());
  return { client, app, order, runAnimationFrame };
}

describe('createLiveApp', () => {
  it('given an app that has not started, when read, then no stream is open and the model has no instrument', () => {
    const { client, app } = setup();

    expect(client.openStreamCount('frames')).toBe(0);
    expect(get(app.model).toolbar.chips[0]?.text).toBe('Connecting');
  });

  it('given an app, when started, then the snapshot is applied and the frame stream is open', async () => {
    const { client, app } = setup();

    await app.start();

    expect(client.openStreamCount('frames')).toBe(1);
    expect(get(app.model).channels[0]?.voltsPerDiv.value).not.toBe('-');
  });

  it('given a started app, when a frame arrives and the animation frame fires, then the graph holds its samples', async () => {
    const { client, app, runAnimationFrame } = setup();
    await app.start();

    client.pushStream('frames', buildFrame({ sequence: 1 }));
    runAnimationFrame();

    expect(get(app.model).graph.channels.length).toBeGreaterThan(0);
  });

  it('given a started app, when an animation frame fires, then the probe sees begin before the draw and end after it', async () => {
    const { client, app, order, runAnimationFrame } = setup();
    await app.start();
    client.pushStream('frames', buildFrame({ sequence: 1 }));

    runAnimationFrame();
    expect(order).toEqual(['begin']);
    await Promise.resolve();

    expect(order).toEqual(['begin', 'end']);
  });

  it('given a started app, when it is stopped, then the frame stream is closed', async () => {
    const { client, app } = setup();
    await app.start();

    app.stop();

    expect(client.openStreamCount('frames')).toBe(0);
  });
});
