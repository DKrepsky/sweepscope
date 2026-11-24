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
  /** Lines after wrapping, see `screenshotNotes`. */
  noteLines: number;
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
  const notes = { x: MARGIN, y: notesY, width: GRAPH_SIZE.width, height: content.noteLines * NOTE_LINE };
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

export const NOTE_SIZE = 11;
// Wide estimate of an average glyph, so a wrapped line always fits without measuring the font.
const NOTE_GLYPH_WIDTH = NOTE_SIZE * 0.62;
const NOTE_MAX_CHARS = Math.floor(GRAPH_SIZE.width / NOTE_GLYPH_WIDTH);

function wrap(note: string): string[] {
  const lines: string[] = [];
  let line = '';
  for (const word of note.split(/\s+/).filter(Boolean)) {
    const next = line ? `${line} ${word}` : word;
    if (next.length > NOTE_MAX_CHARS && line) {
      lines.push(line);
      line = word;
    } else {
      line = next;
    }
  }
  return line ? [...lines, line] : lines;
}

export interface ScreenshotNotes {
  footnotes: string[];
  accuracy: string[];
}

/** Notes wrapped to the picture width so no line runs past the right edge. */
export const screenshotNotes = (footnotes: readonly string[], accuracyNote: string): ScreenshotNotes => ({
  footnotes: footnotes.flatMap(wrap),
  accuracy: wrap(accuracyNote),
});

export const noteLineCount = (notes: ScreenshotNotes): number => notes.footnotes.length + notes.accuracy.length;

export const graphPixelSize = (ratio: number): PixelSize => ({ ...GRAPH_SIZE, ratio });
