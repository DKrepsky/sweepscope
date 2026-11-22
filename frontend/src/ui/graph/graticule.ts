import { tokens } from '../../design-system';
import type { PixelSize } from './graph-frame';

export const DIVISIONS_X = 10;
export const DIVISIONS_Y = 8;
const TICKS_PER_DIVISION = 5;
const TICK_LENGTH_CSS = 3;
const EPSILON = 1e-6;

export const deviceExtent = (cssLength: number, ratio: number) => Math.round(cssLength * ratio);

/** CSS coordinate of the centre of a device pixel: lines placed here are exactly one device pixel crisp. */
export const crisp = (devicePixel: number, ratio: number) => (devicePixel + 0.5) / ratio;

export const hairline = (cssWidth: number, ratio: number) => Math.max(1, Math.round(cssWidth * ratio)) / ratio;

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

export function drawGraticule(ctx: CanvasRenderingContext2D, size: PixelSize) {
  const width = hairline(1, size.ratio);
  ctx.fillStyle = tokens['color-bg-scope'];
  ctx.fillRect(0, 0, size.width, size.height);
  strokeLines(ctx, tokens['color-grid-major'], width, majorLines(size));
  strokeLines(ctx, tokens['color-grid-tick'], width, tickLines(size));
  strokeLines(ctx, tokens['color-grid-axis'], width, axisLines(size));
  strokeLines(ctx, tokens['color-grid-frame'], width, frameLines(size));
}
