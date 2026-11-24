import type { GraphFrame, PixelSize, TraceRenderer } from './graph-frame';
import { drawGraticule } from './graticule';
import { PersistenceLayer } from './persistence-layer';
import type { SurfaceFactory } from './persistence-layer';
import { drawTraces } from './traces';

const sizeKey = (size: PixelSize, frame: GraphFrame) =>
  `${size.width}x${size.height}@${size.ratio}|${JSON.stringify([frame.ticksX, frame.ticksY])}`;

/** Graticule layer is redrawn only when the size, ratio or tick labels change; ghosts and traces on every frame. */
export class Canvas2dRenderer implements TraceRenderer {
  private drawnGraticule: string | null = null;
  private readonly ghosts: PersistenceLayer | null;

  constructor(
    private readonly graticule: CanvasRenderingContext2D,
    private readonly traces: CanvasRenderingContext2D,
    ghosts?: CanvasRenderingContext2D,
    createSurface?: SurfaceFactory,
  ) {
    this.ghosts = ghosts ? new PersistenceLayer(ghosts, createSurface) : null;
  }

  draw(frame: GraphFrame, size: PixelSize): void {
    const key = sizeKey(size, frame);
    if (key !== this.drawnGraticule) {
      this.graticule.setTransform(size.ratio, 0, 0, size.ratio, 0, 0);
      drawGraticule(this.graticule, size, { x: frame.ticksX, y: frame.ticksY });
      this.drawnGraticule = key;
    }
    this.ghosts?.draw(frame, size);
    drawTraces(this.traces, frame, size);
  }

  dispose(): void {
    this.drawnGraticule = null;
    this.ghosts?.dispose();
  }
}
