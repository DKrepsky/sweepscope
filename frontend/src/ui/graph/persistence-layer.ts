import type { GraphFrame, PersistenceOptions, PixelSize } from './graph-frame';
import { drawGhostTraces } from './traces';

export const BUCKETS = 8;
/** Design 5.2 opacities (tokens opacity-persist-*); the newest ghost is never brighter than the live trace. */
export const OPACITY_NEWEST = 0.85;
export const OPACITY_OLDEST = 0.3;
export const OPACITY_HOLD = 0.6;
const FROZEN_OPACITY = 0.4;

export interface Surface {
  image: CanvasImageSource;
  ctx: CanvasRenderingContext2D;
}

export type SurfaceFactory = (width: number, height: number) => Surface;

export const canvasSurface: SurfaceFactory = (width, height) => {
  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  return { image: canvas, ctx: canvas.getContext('2d')! };
};

interface Bucket {
  surface: Surface;
  slot: number;
}

/** Opacity of the bucket that is `age` steps old (0 = newest), linear down to the oldest one. */
export const bucketOpacity = (age: number) =>
  OPACITY_NEWEST + ((OPACITY_OLDEST - OPACITY_NEWEST) * age) / (BUCKETS - 1);

function slotOf({ decayMs, now }: PersistenceOptions): number {
  return decayMs === 'infinite' ? 0 : Math.floor(now() / (decayMs / BUCKETS));
}

const settingsKey = (p: PersistenceOptions, frame: GraphFrame, size: PixelSize) =>
  [size.width, size.height, size.ratio, p.decayMs, p.epoch, p.revision ?? 0, frame.dots ? 'dots' : 'line'].join('|');

/**
 * Ghosts live in eight CSS-resolution surfaces, one per age slot; each frame is drawn once into the surface of its
 * slot and every surface is blitted once per draw, so overlap brightening only happens between slots.
 */
export class PersistenceLayer {
  private buckets: (Bucket | undefined)[] = [];
  private key = '';
  private lastChannels: GraphFrame['channels'] | undefined;
  private shown = false;

  constructor(
    private readonly ctx: CanvasRenderingContext2D,
    private readonly createSurface: SurfaceFactory = canvasSurface,
  ) {}

  draw(frame: GraphFrame, size: PixelSize): void {
    const options = frame.persistence;
    if (!options || frame.xy || frame.state === 'empty') return this.release(size);
    const key = settingsKey(options, frame, size);
    if (key !== this.key) this.reset(key);
    const slot = slotOf(options);
    if (frame.channels !== this.lastChannels) this.accumulate(frame, size, slot);
    this.lastChannels = frame.channels;
    this.composite(options, frame, size, slot);
  }

  dispose(): void {
    this.buckets = [];
    this.key = '';
    this.lastChannels = undefined;
  }

  private release(size: PixelSize) {
    if (this.shown) {
      this.ctx.setTransform(size.ratio, 0, 0, size.ratio, 0, 0);
      this.ctx.clearRect(0, 0, size.width, size.height);
    }
    this.shown = false;
    this.dispose();
  }

  private reset(key: string) {
    this.buckets = [];
    this.lastChannels = undefined;
    this.key = key;
  }

  private bucketFor(slot: number, size: PixelSize): Bucket {
    const index = slot % BUCKETS;
    let bucket = this.buckets[index];
    if (!bucket) {
      bucket = { surface: this.createSurface(size.width, size.height), slot };
      this.buckets[index] = bucket;
    }
    if (bucket.slot !== slot) {
      bucket.surface.ctx.clearRect(0, 0, size.width, size.height);
      bucket.slot = slot;
    }
    return bucket;
  }

  private accumulate(frame: GraphFrame, size: PixelSize, slot: number) {
    const { ctx } = this.bucketFor(slot, size).surface;
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    drawGhostTraces(ctx, frame, { ...size, ratio: 1 });
  }

  private composite(options: PersistenceOptions, frame: GraphFrame, size: PixelSize, slot: number) {
    const { ctx } = this;
    ctx.setTransform(size.ratio, 0, 0, size.ratio, 0, 0);
    ctx.clearRect(0, 0, size.width, size.height);
    const dim = frame.state === 'frozen' ? FROZEN_OPACITY : 1;
    for (const bucket of this.buckets) {
      if (!bucket) continue;
      const age = slot - bucket.slot;
      if (age < 0 || age >= BUCKETS) continue;
      ctx.globalAlpha = dim * (options.decayMs === 'infinite' ? OPACITY_HOLD : bucketOpacity(age));
      ctx.drawImage(bucket.surface.image, 0, 0, size.width, size.height);
    }
    ctx.globalAlpha = 1;
    this.shown = true;
  }
}
