import type { GraphFrame, PixelSize, TraceRenderer } from './graph-frame';
import { drawGraticule } from './graticule';
import { drawTraces } from './traces';

const sizeKey = (size: PixelSize) => `${size.width}x${size.height}@${size.ratio}`;

/** Graticule layer is redrawn only when the size or ratio changes; traces layer on every frame. */
export class Canvas2dRenderer implements TraceRenderer {
  private drawnGraticule: string | null = null;

  constructor(
    private readonly graticule: CanvasRenderingContext2D,
    private readonly traces: CanvasRenderingContext2D,
  ) {}

  draw(frame: GraphFrame, size: PixelSize): void {
    const key = sizeKey(size);
    if (key !== this.drawnGraticule) {
      this.graticule.setTransform(size.ratio, 0, 0, size.ratio, 0, 0);
      drawGraticule(this.graticule, size);
      this.drawnGraticule = key;
    }
    drawTraces(this.traces, frame, size);
  }

  dispose(): void {
    this.drawnGraticule = null;
  }
}
