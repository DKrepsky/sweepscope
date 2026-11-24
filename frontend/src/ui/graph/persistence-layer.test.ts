import { describe, expect, it } from 'vitest';
import { tokens } from '../../design-system';
import { Canvas2dRenderer } from './canvas-2d-renderer';
import type { GraphFrame, PersistenceOptions, PixelSize } from './graph-frame';
import { bucketOpacity, OPACITY_HOLD, OPACITY_NEWEST, OPACITY_OLDEST } from './persistence-layer';
import type { Surface } from './persistence-layer';
import { createRecordingContext } from './recording-context';

const size: PixelSize = { width: 400, height: 320, ratio: 1 };

function setup() {
  const ghosts = createRecordingContext();
  const surfaces: ReturnType<typeof createRecordingContext>[] = [];
  const createSurface = (): Surface => {
    const surface = createRecordingContext();
    surfaces.push(surface);
    return { image: surface as unknown as CanvasImageSource, ctx: surface.ctx };
  };
  const renderer = new Canvas2dRenderer(
    createRecordingContext().ctx,
    createRecordingContext().ctx,
    ghosts.ctx,
    createSurface,
  );
  return { ghosts, surfaces, renderer };
}

function clock(start = 0) {
  let time = start;
  return {
    now: () => time,
    advance: (ms: number) => void (time += ms),
  };
}

const frameOf = (
  persistence: PersistenceOptions | undefined,
  samples = [1, 2, 3],
  state: GraphFrame['state'] = 'live',
) => ({ state, channels: [{ channel: 1, samples }], persistence }) as GraphFrame;

const lastBlit = (ghosts: { images: { alpha: number }[] }, count: number) => ghosts.images.slice(-count);

describe('PersistenceLayer ageing', () => {
  it('given persistence 500 ms and a new frame every 62.5 ms, when drawn, then the ghosts step from 0.85 to 0.30 in eight steps', () => {
    const { ghosts, renderer } = setup();
    const time = clock();
    const options: PersistenceOptions = { decayMs: 500, epoch: 0, now: time.now };
    for (let i = 0; i < 8; i++) {
      renderer.draw(frameOf(options), size);
      time.advance(62.5);
    }
    time.advance(-62.5);
    ghosts.images.length = 0;
    renderer.draw({ ...frameOf(options), channels: frameOf(options).channels }, size);
    const alphas = lastBlit(ghosts, 8).map((i) => i.alpha);
    const expected = Array.from({ length: 8 }, (_, age) => bucketOpacity(age));
    expect(alphas.slice().sort((a, b) => b - a)).toEqual(expected);
    expect(expected[0]).toBeCloseTo(OPACITY_NEWEST, 9);
    expect(expected[7]).toBeCloseTo(OPACITY_OLDEST, 9);
  });

  it('given a ghost drawn at 0 ms and persistence 500 ms, when the clock passes the decay, then the ghost is no longer blitted', () => {
    const { ghosts, renderer } = setup();
    const time = clock();
    const options: PersistenceOptions = { decayMs: 500, epoch: 0, now: time.now };
    const first = frameOf(options);
    renderer.draw(first, size);
    time.advance(500);
    ghosts.images.length = 0;
    renderer.draw(first, size);
    expect(ghosts.images).toEqual([]);
  });

  it('given the clock stands still as when stopped, when the same frame is drawn again, then the ghost opacity does not change', () => {
    const { ghosts, renderer } = setup();
    const time = clock(1000);
    const options: PersistenceOptions = { decayMs: 1000, epoch: 0, now: time.now };
    const frame = frameOf(options);
    renderer.draw(frame, size);
    const first = ghosts.images.at(-1)!.alpha;
    renderer.draw(frame, size);
    expect(ghosts.images.at(-1)!.alpha).toBe(first);
  });

  it('given persistence Infinite, when a long time passes, then the ghost keeps the hold opacity 0.60', () => {
    const { ghosts, renderer } = setup();
    const time = clock();
    const options: PersistenceOptions = { decayMs: 'infinite', epoch: 0, now: time.now };
    renderer.draw(frameOf(options), size);
    time.advance(10 * 60 * 1000);
    renderer.draw(frameOf(options), size);
    expect(ghosts.images.map((i) => i.alpha)).toEqual([OPACITY_HOLD, OPACITY_HOLD]);
  });

  it('given a frame that is blitted, when drawn, then each age slot is blitted once so overlap only brightens between slots', () => {
    const { ghosts, surfaces, renderer } = setup();
    const time = clock();
    const options: PersistenceOptions = { decayMs: 1000, epoch: 0, now: time.now };
    renderer.draw(frameOf(options), size);
    renderer.draw(frameOf(options), size);
    expect(surfaces).toHaveLength(1);
    expect(ghosts.images).toHaveLength(2);
    expect(surfaces[0]!.strokes).toHaveLength(2);
  });
});

describe('PersistenceLayer reset', () => {
  const frames = (renderer: Canvas2dRenderer, options: PersistenceOptions, resized = size) => {
    renderer.draw(frameOf(options), resized);
    renderer.draw(frameOf({ ...options }), resized);
  };

  it('given ghosts, when the epoch changes by Clear, then the old ghosts are dropped and only the new frame is blitted', () => {
    const { ghosts, renderer } = setup();
    const time = clock();
    frames(renderer, { decayMs: 'infinite', epoch: 0, now: time.now });
    ghosts.images.length = 0;
    renderer.draw(frameOf({ decayMs: 'infinite', epoch: 1, now: time.now }), size);
    expect(ghosts.images).toHaveLength(1);
  });

  it('given ghosts, when the setup revision changes, then the ghosts clear', () => {
    const { surfaces, renderer } = setup();
    const time = clock();
    frames(renderer, { decayMs: 'infinite', epoch: 0, revision: 1, now: time.now });
    renderer.draw(frameOf({ decayMs: 'infinite', epoch: 0, revision: 2, now: time.now }), size);
    expect(surfaces).toHaveLength(2);
    expect(surfaces[1]!.strokes).toHaveLength(1);
  });

  it('given ghosts, when the screen is resized, then the ghosts clear', () => {
    const { surfaces, renderer } = setup();
    const options: PersistenceOptions = { decayMs: 'infinite', epoch: 0, now: clock().now };
    frames(renderer, options);
    renderer.draw(frameOf(options), { ...size, width: 300 });
    expect(surfaces).toHaveLength(2);
  });

  it('given dots turned on, when drawn, then the ghosts clear (interpolation change)', () => {
    const { surfaces, renderer } = setup();
    const options: PersistenceOptions = { decayMs: 'infinite', epoch: 0, now: clock().now };
    frames(renderer, options);
    renderer.draw({ ...frameOf(options), dots: true }, size);
    expect(surfaces).toHaveLength(2);
  });

  it('given persistence turned off, when drawn, then the ghost layer is cleared and nothing is blitted', () => {
    const { ghosts, renderer } = setup();
    const options: PersistenceOptions = { decayMs: 'infinite', epoch: 0, now: clock().now };
    renderer.draw(frameOf(options), size);
    ghosts.images.length = 0;
    renderer.draw(frameOf(undefined), size);
    renderer.draw(frameOf(undefined), size);
    expect(ghosts.images).toEqual([]);
  });

  it('given an XY frame with persistence set, when drawn, then no ghosts are blitted', () => {
    const { ghosts, renderer } = setup();
    const options: PersistenceOptions = { decayMs: 'infinite', epoch: 0, now: clock().now };
    const xy = { x: [1, 2], y: [1, 2], xChannel: 1, yChannel: 2 } as const;
    renderer.draw({ ...frameOf(options), xy }, size);
    expect(ghosts.images).toEqual([]);
  });
});

describe('PersistenceLayer glitch and state', () => {
  it('given a one-sample glitch in 2501 points at 512 px, when the ghost is made, then its path reaches the glitch column in the channel colour', () => {
    const { surfaces, renderer } = setup();
    const samples = Array.from({ length: 2501 }, () => 4);
    samples[1250] = 0.5;
    const options: PersistenceOptions = { decayMs: 1000, epoch: 0, now: clock().now };
    renderer.draw(frameOf(options, samples), { width: 512, height: 400, ratio: 1 });
    const stroke = surfaces[0]!.strokes[0]!;
    const glitchX = (1250 / 2500) * 512;
    expect(stroke.style).toBe(tokens['color-ch1']);
    expect(stroke.points.some(([x, y]) => x === glitchX && y < 100)).toBe(true);
    expect(stroke.width).toBeGreaterThanOrEqual(1);
  });

  it('given a frozen frame, when ghosts are blitted, then they are dimmed to 40 %', () => {
    const { ghosts, renderer } = setup();
    const options: PersistenceOptions = { decayMs: 'infinite', epoch: 0, now: clock().now };
    renderer.draw(frameOf(options, [1, 2], 'frozen'), size);
    expect(ghosts.images[0]!.alpha).toBeCloseTo(OPACITY_HOLD * 0.4, 9);
  });
});
