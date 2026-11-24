import type { PixelSize } from '../graph/graph-frame';

export const MARGIN = 24;
export const GRAPH_SIZE = { width: 880, height: 704 };
export const HEADER_LINE = 18;
export const TABLE_ROW = 22;
export const NOTE_LINE = 16;
const GAP = 16;
const LABEL_COLUMN = 160;

export interface ScreenshotContent {
  header: readonly string[];
  tableRows: number;
  footnotes: number;
}

export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface ScreenshotLayout {
  size: { width: number; height: number };
  header: Rect;
  graph: Rect;
  table: Rect;
  notes: Rect;
  labelColumn: number;
}

/** Everything in css pixels, stacked top to bottom: header, screen, table, notes. */
export function screenshotLayout(content: ScreenshotContent): ScreenshotLayout {
  const width = GRAPH_SIZE.width + 2 * MARGIN;
  const header = { x: MARGIN, y: MARGIN, width: GRAPH_SIZE.width, height: content.header.length * HEADER_LINE };
  const graph = { x: MARGIN, y: header.y + header.height + GAP, ...GRAPH_SIZE };
  const tableHeight = content.tableRows > 0 ? (content.tableRows + 1) * TABLE_ROW : 0;
  const table = { x: MARGIN, y: graph.y + graph.height + GAP, width: GRAPH_SIZE.width, height: tableHeight };
  const notesY = table.y + tableHeight + (tableHeight > 0 ? GAP : 0);
  const notes = { x: MARGIN, y: notesY, width: GRAPH_SIZE.width, height: (content.footnotes + 1) * NOTE_LINE };
  return {
    size: { width, height: notes.y + notes.height + MARGIN },
    header,
    graph,
    table,
    notes,
    labelColumn: LABEL_COLUMN,
  };
}

export const screenshotSize = (content: ScreenshotContent): { width: number; height: number } =>
  screenshotLayout(content).size;

export const graphPixelSize = (ratio: number): PixelSize => ({ ...GRAPH_SIZE, ratio });
