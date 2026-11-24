import { tokens, type TokenName } from '../../design-system';
import { crisp, deviceExtent, drawGraticule } from '../graph/graticule';
import type { GraphFrame } from '../graph/graph-frame';
import { drawTraces } from '../graph/traces';
import {
  graphPixelSize,
  screenshotLayout,
  screenshotNotes,
  HEADER_LINE,
  NOTE_LINE,
  NOTE_SIZE,
  TABLE_ROW,
  type Rect,
} from './screenshot-layout';

export interface ScreenshotMarker {
  axis: 'x' | 'y';
  /** Fraction of the screen: x from the left, y from the top. */
  position: number;
  tag: string;
}

export interface ScreenshotCell {
  /** Already formatted, footnote mark included. */
  text: string;
  /** The cell has no value. */
  muted?: boolean;
}

export interface ScreenshotTable {
  /** `color` is a token name or its css variable form. */
  columns: { label: string; color: string }[];
  rows: { label: string; cells: ScreenshotCell[] }[];
}

/** Plain, already formatted data: the page composes it from the window model. */
export interface ScreenshotSpec {
  graph: GraphFrame;
  header: string[];
  cursors: ScreenshotMarker[];
  triggerMarkers: ScreenshotMarker[];
  table: ScreenshotTable;
  footnotes: string[];
  frozenLabel?: string;
}

const SANS = tokens['font-ui'];
const MONO = tokens['font-mono'];
const CURSOR_DASH = [6, 4];
const TAG_PADDING = 4;
const TEXT_SIZE = 12;
const BOLD = 650;

function resolveColour(reference: string): string {
  const name = reference.replace(/^var\(--ss-/, '').replace(/\)$/, '');
  return tokens[name as TokenName] ?? tokens['color-text-primary'];
}

interface TextStyle {
  colour: string;
  font?: string;
  size?: number;
  weight?: number;
  align?: CanvasTextAlign;
}

function text(ctx: CanvasRenderingContext2D, value: string, x: number, y: number, style: TextStyle) {
  ctx.fillStyle = style.colour;
  ctx.font = `${style.weight ?? 400} ${style.size ?? TEXT_SIZE}px ${style.font ?? SANS}`;
  ctx.textAlign = style.align ?? 'left';
  ctx.textBaseline = 'middle';
  ctx.fillText(value, x, y);
}

function drawHeader(ctx: CanvasRenderingContext2D, lines: string[], area: Rect) {
  lines.forEach((line, index) => {
    const title = index === 0;
    text(ctx, line, area.x, area.y + index * HEADER_LINE + HEADER_LINE / 2, {
      colour: tokens[title ? 'color-text-primary' : 'color-text-secondary'],
      font: title ? SANS : MONO,
      weight: title ? BOLD : 400,
    });
  });
}

/** Lets the shared graph drawing, which resets the transform and clears the canvas, paint inside a window of it. */
function inWindow(ctx: CanvasRenderingContext2D, dx: number, dy: number): CanvasRenderingContext2D {
  return new Proxy(ctx, {
    get(target, key) {
      if (key === 'setTransform') {
        return (a: number, b: number, c: number, d: number, e: number, f: number) => {
          target.setTransform(a, b, c, d, e + dx, f + dy);
        };
      }
      if (key === 'clearRect') return () => {};
      const value = Reflect.get(target, key) as unknown;
      return typeof value === 'function' ? (value as () => void).bind(target) : value;
    },
    set: (target, key, value) => Reflect.set(target, key, value),
  });
}

const TAG_CLEARANCE = TAG_PADDING + 2;

/** Vertical centre of a label for a horizontal marker: above the line, or below it when that would sit on the centre axis. */
export function horizontalTagY(lineY: number, graphHeight: number): number {
  const axis = graphHeight / 2;
  const half = TEXT_SIZE / 2;
  const above = lineY - TAG_CLEARANCE - half;
  const below = lineY + TAG_CLEARANCE + half;
  const touchesAxis = (centre: number) => Math.abs(centre - axis) < half + 1;
  if (!touchesAxis(above) && above - half >= 0) return above;
  return touchesAxis(below) ? above : below;
}

function drawMarker(ctx: CanvasRenderingContext2D, marker: ScreenshotMarker, colour: string, ratio: number) {
  const { width, height } = graphPixelSize(ratio);
  const vertical = marker.axis === 'x';
  const extent = deviceExtent(vertical ? width : height, ratio);
  const at = crisp(Math.min(extent - 1, Math.max(0, Math.floor(marker.position * extent))), ratio);
  ctx.strokeStyle = colour;
  ctx.lineWidth = 1 / ratio;
  ctx.setLineDash(CURSOR_DASH);
  ctx.beginPath();
  ctx.moveTo(vertical ? at : 0, vertical ? 0 : at);
  ctx.lineTo(vertical ? at : width, vertical ? height : at);
  ctx.stroke();
  ctx.setLineDash([]);
  const x = vertical ? at + TAG_PADDING : TAG_PADDING;
  const y = vertical ? TAG_PADDING + TEXT_SIZE / 2 : horizontalTagY(at, height);
  text(ctx, marker.tag, x, y, { colour, font: MONO, weight: BOLD });
}

function drawGraph(ctx: CanvasRenderingContext2D, spec: ScreenshotSpec, area: Rect, ratio: number) {
  const size = graphPixelSize(ratio);
  // The window starts on a whole device pixel so the graticule hairlines stay exactly one pixel wide.
  const dx = Math.round(area.x * ratio);
  const dy = Math.round(area.y * ratio);
  ctx.save();
  ctx.setTransform(ratio, 0, 0, ratio, dx, dy);
  drawGraticule(ctx, size);
  drawTraces(inWindow(ctx, dx, dy), spec.graph, size);
  for (const marker of spec.cursors) drawMarker(ctx, marker, tokens['color-cursor'], ratio);
  for (const marker of spec.triggerMarkers) drawMarker(ctx, marker, tokens['color-trigger'], ratio);
  if (spec.frozenLabel) {
    text(ctx, spec.frozenLabel, size.width - TAG_PADDING * 2, TAG_PADDING * 3, {
      colour: tokens['color-status-warning'],
      weight: BOLD,
      align: 'right',
    });
  }
  ctx.restore();
}

function drawTable(ctx: CanvasRenderingContext2D, table: ScreenshotTable, area: Rect, labelColumn: number) {
  if (table.rows.length === 0) return;
  const columnWidth = (area.width - labelColumn) / Math.max(1, table.columns.length);
  const centre = (row: number) => area.y + row * TABLE_ROW + TABLE_ROW / 2;
  const right = (column: number) => area.x + labelColumn + (column + 1) * columnWidth - TAG_PADDING * 2;
  table.columns.forEach((column, index) => {
    text(ctx, column.label, right(index), centre(0), {
      colour: resolveColour(column.color),
      weight: BOLD,
      align: 'right',
    });
  });
  table.rows.forEach((row, rowIndex) => {
    text(ctx, row.label, area.x, centre(rowIndex + 1), { colour: tokens['color-text-secondary'] });
    row.cells.forEach((cell, index) => {
      const colour = tokens[cell.muted ? 'color-text-muted' : 'color-text-primary'];
      text(ctx, cell.text, right(index), centre(rowIndex + 1), { colour, font: MONO, align: 'right' });
    });
  });
}

function drawNotes(ctx: CanvasRenderingContext2D, spec: ScreenshotSpec, area: Rect) {
  screenshotNotes(spec.footnotes).forEach((line, index) => {
    const colour = tokens['color-text-secondary'];
    text(ctx, line, area.x, area.y + index * NOTE_LINE + NOTE_LINE / 2, { colour, size: NOTE_SIZE });
  });
}

function layoutOf(spec: ScreenshotSpec) {
  return screenshotLayout({
    header: spec.header,
    tableRows: spec.table.rows.length,
    noteLines: screenshotNotes(spec.footnotes).length,
  });
}

/** Css size of the picture for this spec; the canvas is this times the ratio. */
export const specSize = (spec: ScreenshotSpec): { width: number; height: number } => layoutOf(spec).size;

/** Paints the whole picture into a canvas context of css size (`specSize`) times `ratio`. */
export function drawScreenshot(ctx: CanvasRenderingContext2D, spec: ScreenshotSpec, ratio: number): void {
  const layout = layoutOf(spec);
  ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
  ctx.fillStyle = tokens['color-bg-app'];
  ctx.fillRect(0, 0, layout.size.width, layout.size.height);
  drawHeader(ctx, spec.header, layout.header);
  drawGraph(ctx, spec, layout.graph, ratio);
  drawTable(ctx, spec.table, layout.table, layout.labelColumn);
  drawNotes(ctx, spec, layout.notes);
}
