import { tokens } from '../../design-system';
import type { ChannelNumber, GraphFrame, PixelSize, Trace } from './graph-frame';
import { DIVISIONS_Y } from './graticule';

const FROZEN_OPACITY = 0.4;
const DOT_SIZE_CSS = 3;
const EDGE_LINE_CSS = 2;
const TRIANGLE_CSS = 5;
const DASH: number[] = [6, 4];

interface Layer {
  trace: Trace;
  colour: string;
  width: number;
  dash: number[];
  channel?: ChannelNumber;
}

const cssPx = (token: string) => Number.parseFloat(token);

function layersBottomToTop(frame: GraphFrame): Layer[] {
  const layers: Layer[] = [];
  if (frame.reference) layers.push({ trace: frame.reference, colour: tokens['color-ref'], width: 1, dash: DASH });
  if (frame.math) layers.push({ trace: frame.math, colour: tokens['color-math'], width: 1, dash: [] });
  const drawnLast = (c: { channel: ChannelNumber }) => (c.channel === frame.selected ? 0 : c.channel);
  const channels = [...frame.channels].sort((a, b) => drawnLast(b) - drawnLast(a));
  for (const { channel, samples } of channels) {
    const colour = tokens[`color-ch${channel}`];
    layers.push({ trace: { samples }, colour, width: cssPx(tokens['trace-width']), dash: [], channel });
  }
  return layers;
}

function point(size: PixelSize, samples: ArrayLike<number>, index: number): [number, number] {
  const x = samples.length > 1 ? (index / (samples.length - 1)) * size.width : size.width / 2;
  const clamped = Math.min(DIVISIONS_Y, Math.max(0, samples[index] ?? 0));
  return [x, (clamped / DIVISIONS_Y) * size.height];
}

const side = (value: number) => (value < 0 ? 'top' : value > DIVISIONS_Y ? 'bottom' : null);

function strokeLine(ctx: CanvasRenderingContext2D, layer: Layer, size: PixelSize) {
  const { samples } = layer.trace;
  ctx.strokeStyle = layer.colour;
  ctx.lineWidth = Math.max(1 / size.ratio, layer.width);
  ctx.lineJoin = 'round';
  ctx.setLineDash(layer.dash);
  ctx.beginPath();
  for (let i = 0; i < samples.length; i++) {
    const [x, y] = point(size, samples, i);
    if (i === 0) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);
  }
  ctx.stroke();
}

function fillDots(ctx: CanvasRenderingContext2D, layer: Layer, size: PixelSize) {
  ctx.fillStyle = layer.colour;
  const { samples } = layer.trace;
  for (let i = 0; i < samples.length; i++) {
    const [x, y] = point(size, samples, i);
    ctx.fillRect(x - DOT_SIZE_CSS / 2, y - DOT_SIZE_CSS / 2, DOT_SIZE_CSS, DOT_SIZE_CSS);
  }
}

function offScreenRuns(samples: ArrayLike<number>) {
  const runs: { edge: 'top' | 'bottom'; first: number; last: number }[] = [];
  for (let i = 0; i < samples.length; i++) {
    const edge = side(samples[i] ?? 0);
    const open = runs.at(-1);
    if (!edge) continue;
    if (open && open.edge === edge && open.last === i - 1) open.last = i;
    else runs.push({ edge, first: i, last: i });
  }
  return runs;
}

function markOffScreen(ctx: CanvasRenderingContext2D, layer: Layer, size: PixelSize) {
  const { samples } = layer.trace;
  ctx.fillStyle = layer.colour;
  ctx.strokeStyle = layer.colour;
  ctx.lineWidth = Math.max(1 / size.ratio, EDGE_LINE_CSS);
  ctx.setLineDash([]);
  for (const run of offScreenRuns(samples)) {
    const [x1, y] = point(size, samples, run.first);
    const [x2] = point(size, samples, run.last);
    ctx.beginPath();
    ctx.moveTo(x1, y);
    ctx.lineTo(x2, y);
    ctx.stroke();
    triangle(ctx, (x1 + x2) / 2, y, run.edge === 'top' ? -1 : 1);
  }
}

function triangle(ctx: CanvasRenderingContext2D, x: number, edgeY: number, direction: -1 | 1) {
  const inward = edgeY - direction * TRIANGLE_CSS;
  ctx.beginPath();
  ctx.moveTo(x, edgeY);
  ctx.lineTo(x - TRIANGLE_CSS, inward);
  ctx.lineTo(x + TRIANGLE_CSS, inward);
  ctx.closePath();
  ctx.fill();
}

function drawLayer(ctx: CanvasRenderingContext2D, layer: Layer, frame: GraphFrame, size: PixelSize) {
  if (frame.dots && layer.channel) fillDots(ctx, layer, size);
  else strokeLine(ctx, layer, size);
  if (layer.channel) markOffScreen(ctx, layer, size);
}

export function drawTraces(ctx: CanvasRenderingContext2D, frame: GraphFrame, size: PixelSize) {
  ctx.setTransform(size.ratio, 0, 0, size.ratio, 0, 0);
  ctx.clearRect(0, 0, size.width, size.height);
  if (frame.state === 'empty') return;
  ctx.save();
  ctx.globalAlpha = frame.state === 'frozen' ? FROZEN_OPACITY : 1;
  for (const layer of layersBottomToTop(frame)) drawLayer(ctx, layer, frame, size);
  ctx.restore();
}
