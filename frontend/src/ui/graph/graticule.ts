import { tokens } from '../../design-system';
import type { PixelSize, Tick } from './graph-frame';

export const DIVISIONS_X = 10;
export const DIVISIONS_Y = 8;
const TICKS_PER_DIVISION = 5;
const TICK_LENGTH_CSS = 3;
const EPSILON = 1e-6;

export const deviceExtent = (cssLength: number, ratio: number) => Math.round(cssLength * ratio);

/** CSS coordinate of the centre of a device pixel: lines placed here are exactly one device pixel crisp. */
export const crisp = (devicePixel: number, ratio: number) => (devicePixel + 0.5) / ratio;

/** Graticule lines are exactly one device pixel wide at every ratio (design 8.5), never a rounded CSS width. */
export const hairline = (ratio: number) => 1 / ratio;

interface Line {
  from: [number, number];
  to: [number, number];
}

/** Crisp CSS coordinates of the given step indices along one axis split into `steps` equal parts. */
function position(cssLength: number, ratio: number, steps: number, index: number): number {
  return crisp(Math.floor((index / steps) * deviceExtent(cssLength, ratio) + EPSILON), ratio);
}

const positions = (cssLength: number, ratio: number, steps: number, indices: number[]) =>
  indices.map((i) => position(cssLength, ratio, steps, i));

const range = (first: number, last: number) => Array.from({ length: last - first + 1 }, (_, i) => first + i);

function verticals(size: PixelSize, steps: number, indices: number[], span: [number, number]): Line[] {
  return positions(size.width, size.ratio, steps, indices).map((x) => ({ from: [x, span[0]], to: [x, span[1]] }));
}

function horizontals(size: PixelSize, steps: number, indices: number[], span: [number, number]): Line[] {
  return positions(size.height, size.ratio, steps, indices).map((y) => ({ from: [span[0], y], to: [span[1], y] }));
}

function majorLines(size: PixelSize): Line[] {
  return [
    ...verticals(size, DIVISIONS_X, range(1, DIVISIONS_X - 1), [0, size.height]),
    ...horizontals(size, DIVISIONS_Y, range(1, DIVISIONS_Y - 1), [0, size.width]),
  ];
}

function axisLines(size: PixelSize): Line[] {
  return [...verticals(size, 2, [1], [0, size.height]), ...horizontals(size, 2, [1], [0, size.width])];
}

function tickLines(size: PixelSize): Line[] {
  const axisX = position(size.width, size.ratio, 2, 1);
  const axisY = position(size.height, size.ratio, 2, 1);
  const xTicks = DIVISIONS_X * TICKS_PER_DIVISION;
  const yTicks = DIVISIONS_Y * TICKS_PER_DIVISION;
  return [
    ...verticals(size, xTicks, range(1, xTicks - 1), [axisY - TICK_LENGTH_CSS, axisY + TICK_LENGTH_CSS]),
    ...horizontals(size, yTicks, range(1, yTicks - 1), [axisX - TICK_LENGTH_CSS, axisX + TICK_LENGTH_CSS]),
  ];
}

function strokeLines(ctx: CanvasRenderingContext2D, style: string, width: number, lines: Line[]) {
  ctx.strokeStyle = style;
  ctx.lineWidth = width;
  ctx.beginPath();
  for (const { from, to } of lines) {
    ctx.moveTo(...from);
    ctx.lineTo(...to);
  }
  ctx.stroke();
}

function frameLines(size: PixelSize): Line[] {
  const { ratio } = size;
  const left = crisp(0, ratio);
  const top = crisp(0, ratio);
  const right = crisp(deviceExtent(size.width, ratio) - 1, ratio);
  const bottom = crisp(deviceExtent(size.height, ratio) - 1, ratio);
  return [
    { from: [left, top], to: [right, top] },
    { from: [right, top], to: [right, bottom] },
    { from: [right, bottom], to: [left, bottom] },
    { from: [left, bottom], to: [left, top] },
  ];
}

export interface GraticuleTicks {
  x?: readonly Tick[];
  y?: readonly Tick[];
}

const EDGE_BAND = 0.03;

function anchor<T extends string>(at: number, low: T, middle: T, high: T): T {
  if (at <= EDGE_BAND) return low;
  return at >= 1 - EDGE_BAND ? high : middle;
}

const within = (value: number, extent: number, inset: number) => Math.min(extent - inset, Math.max(inset, value));

/** Labels sit inside the frame, so no gutter is needed; the ones at an edge are anchored inward to stay inside. */
function drawTickLabels(
  ctx: CanvasRenderingContext2D,
  size: PixelSize,
  ticksX: readonly Tick[],
  ticksY: readonly Tick[],
) {
  const inset = Number.parseFloat(tokens['space-2']) + 1;
  ctx.fillStyle = tokens['color-text-muted'];
  ctx.font = `${tokens['font-size-2xs']} ${tokens['font-mono']}`;
  ctx.textBaseline = 'bottom';
  for (const { at, text } of ticksX) {
    ctx.textAlign = anchor<CanvasTextAlign>(at, 'left', 'center', 'right');
    ctx.fillText(text, within(at * size.width, size.width, inset), size.height - inset);
  }
  ctx.textAlign = 'left';
  for (const { at, text } of ticksY) {
    ctx.textBaseline = anchor<CanvasTextBaseline>(at, 'bottom', 'middle', 'top');
    ctx.fillText(text, inset, within((1 - at) * size.height, size.height, inset));
  }
}

export function drawGraticule(ctx: CanvasRenderingContext2D, size: PixelSize, ticks: GraticuleTicks = {}) {
  const width = hairline(size.ratio);
  ctx.fillStyle = tokens['color-bg-scope'];
  ctx.fillRect(0, 0, size.width, size.height);
  strokeLines(ctx, tokens['color-grid-major'], width, majorLines(size));
  strokeLines(ctx, tokens['color-grid-tick'], width, tickLines(size));
  strokeLines(ctx, tokens['color-grid-axis'], width, axisLines(size));
  strokeLines(ctx, tokens['color-grid-frame'], width, frameLines(size));
  drawTickLabels(ctx, size, ticks.x ?? [], ticks.y ?? []);
}
