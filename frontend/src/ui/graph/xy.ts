import { tokens } from '../../design-system';
import type { PixelSize, XyTrace } from './graph-frame';
import { DIVISIONS_X, DIVISIONS_Y } from './graticule';
import { strokeStyled, tracePath } from './path';

const clamp01 = (value: number) => Math.min(1, Math.max(0, value));

/** Divisions are square: X spans 8 of the 10 columns around the centre, like Y spans the 8 rows. */
export function xyPoint(size: PixelSize, xy: XyTrace, index: number): [number, number] {
  const x = clamp01(0.5 + (DIVISIONS_Y / 2 - (xy.x[index] ?? Number.NaN)) / DIVISIONS_X);
  const y = clamp01((xy.y[index] ?? Number.NaN) / DIVISIONS_Y);
  return [x * size.width, y * size.height];
}

/** Straight segments between the points display sent, in the colour of the Y channel. */
export function drawXy(ctx: CanvasRenderingContext2D, xy: XyTrace, size: PixelSize) {
  const colour = tokens[`color-ch${xy.yChannel}`];
  strokeStyled(ctx, colour, Math.max(1 / size.ratio, Number.parseFloat(tokens['trace-width'])), []);
  tracePath(ctx, Math.min(xy.x.length, xy.y.length), (i) => xyPoint(size, xy, i));
  ctx.stroke();
}
