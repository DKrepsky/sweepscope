import { describe, expect, it } from 'vitest';
import { tokens } from '../../design-system';
import { createRecordingContext } from '../graph/recording-context';
import { drawScreenshot, horizontalTagY, specSize, type ScreenshotSpec } from './draw-screenshot';
import { GRAPH_SIZE, screenshotLayout, screenshotNotes, screenshotSize } from './screenshot-layout';

function textRecordingContext() {
  const recording = createRecordingContext();
  const texts: string[] = [];
  const transforms: number[][] = [];
  Object.assign(recording.ctx, {
    fillText: (value: string) => texts.push(value),
    setTransform: (...matrix: number[]) => transforms.push(matrix),
    font: '',
    textAlign: '',
    textBaseline: '',
  });
  return { ...recording, texts, transforms };
}

const spec = (overrides: Partial<ScreenshotSpec> = {}): ScreenshotSpec => ({
  graph: {
    state: 'live',
    channels: [
      { channel: 1, samples: [4, 2, 4, 6] },
      { channel: 2, samples: [3, 3, 3, 3] },
    ],
    selected: 1,
  },
  header: ['SweepScope', 'CH1  500 mV/div'],
  cursors: [],
  triggerMarkers: [],
  table: {
    columns: [{ label: 'CH1', color: 'color-ch1' }],
    rows: [{ label: 'Frequency', cells: [{ text: '1.00 kHz' }] }],
  },
  footnotes: ['[1] Not measurable'],
  accuracyNote: 'Volts accurate to 2 counts',
  ...overrides,
});

describe('drawScreenshot', () => {
  it('given a model when drawn then the header, table and notes text is present', () => {
    const { ctx, texts } = textRecordingContext();

    drawScreenshot(ctx, spec(), 2);

    expect(texts).toEqual(
      expect.arrayContaining(['SweepScope', 'CH1  500 mV/div', 'CH1', 'Frequency', '1.00 kHz', '[1] Not measurable']),
    );
    expect(texts).toContain('Volts accurate to 2 counts');
  });

  it('given two channels when drawn then each has a trace in its own colour', () => {
    const { ctx, strokes } = textRecordingContext();

    drawScreenshot(ctx, spec(), 1);

    const colours = strokes.filter((s) => s.points.length === 4).map((s) => s.style);
    expect(colours).toEqual(expect.arrayContaining([tokens['color-ch1'], tokens['color-ch2']]));
  });

  it('given ratio 2 when drawn then graticule lines are one device pixel wide', () => {
    const { ctx, strokes } = textRecordingContext();

    drawScreenshot(ctx, spec(), 2);

    const grid = strokes.filter((s) => s.style === tokens['color-grid-major']);
    expect(grid).not.toHaveLength(0);
    expect(grid.every((s) => s.width === 0.5)).toBe(true);
  });

  it('given ratio 2 when drawn then the graph window starts on a whole device pixel', () => {
    const { ctx, transforms } = textRecordingContext();

    drawScreenshot(ctx, spec(), 2);

    const layout = screenshotLayout({ header: spec().header, tableRows: 1, noteLines: 2 });
    expect(transforms).toContainEqual([2, 0, 0, 2, layout.graph.x * 2, layout.graph.y * 2]);
    expect(transforms.every(([, , , , e = 0, f = 0]) => Number.isInteger(e) && Number.isInteger(f))).toBe(true);
  });

  it('given a cursor when drawn then a dashed line with its tag is drawn', () => {
    const { ctx, strokes, texts } = textRecordingContext();

    drawScreenshot(ctx, spec({ cursors: [{ axis: 'x', position: 0.25, tag: 'X1' }] }), 1);

    expect(strokes.some((s) => s.style === tokens['color-cursor'] && s.dash.length === 2)).toBe(true);
    expect(texts).toContain('X1');
  });

  it('given a frozen label when drawn then it is written on the screen', () => {
    const { ctx, texts } = textRecordingContext();

    drawScreenshot(ctx, spec({ frozenLabel: 'Frozen' }), 1);

    expect(texts).toContain('Frozen');
  });
});

const LONG_NOTE =
  'Voltage readings are accurate to about 2 to 3 counts (about 5 % of a division at extreme positions). ' +
  'Ranges below 50 mV/div and above 1 V/div are unverified. Verified on one unit only.';

describe('screenshot accuracy note', () => {
  it('given a long accuracy note when wrapped then every line fits the picture width', () => {
    const { accuracy } = screenshotNotes([], LONG_NOTE);

    expect(accuracy.length).toBeGreaterThan(1);
    expect(accuracy.join(' ')).toBe(LONG_NOTE);
    for (const line of accuracy) expect(line.length * 11 * 0.5).toBeLessThanOrEqual(GRAPH_SIZE.width);
  });

  it('given a long accuracy note when drawn then no note line is left out and the picture is tall enough', () => {
    const { ctx, texts } = textRecordingContext();
    const long = spec({ accuracyNote: LONG_NOTE });

    drawScreenshot(ctx, long, 1);

    const { accuracy } = screenshotNotes(long.footnotes, LONG_NOTE);
    for (const line of accuracy) expect(texts).toContain(line);
    const layout = screenshotLayout({ header: long.header, tableRows: 1, noteLines: 1 + accuracy.length });
    expect(specSize(long).height).toBe(layout.size.height);
  });

  it('given a short accuracy note when wrapped then it stays on one line', () => {
    expect(screenshotNotes([], 'Volts accurate to 2 counts').accuracy).toEqual(['Volts accurate to 2 counts']);
  });
});

describe('horizontal marker tag', () => {
  const half = 6;
  const axis = GRAPH_SIZE.height / 2;
  const clearOfAxis = (y: number) => Math.abs(y - axis) > half;

  it('given a marker on the centre line when labelled then the label does not touch the centre line', () => {
    expect(clearOfAxis(horizontalTagY(axis, GRAPH_SIZE.height))).toBe(true);
  });

  it('given a marker just below the centre line when labelled then the label is placed clear of the centre line', () => {
    for (const offset of [1, 5, 10, 14]) {
      expect(clearOfAxis(horizontalTagY(axis + offset, GRAPH_SIZE.height))).toBe(true);
    }
  });

  it('given a marker at the top edge when labelled then the label stays inside the screen', () => {
    expect(horizontalTagY(0, GRAPH_SIZE.height) - half).toBeGreaterThanOrEqual(0);
  });

  it('given a trigger level at 0 V when drawn then the T1 label is written clear of the centre line', () => {
    const { ctx } = textRecordingContext();
    const calls: [string, number][] = [];
    Object.assign(ctx, { fillText: (value: string, _x: number, y: number) => calls.push([value, y]) });

    drawScreenshot(ctx, spec({ triggerMarkers: [{ axis: 'y', position: 0.5, tag: 'T1 ↗ 0.00 V' }] }), 1);

    const tag = calls.find(([value]) => value.startsWith('T1'));
    expect(tag && clearOfAxis(tag[1])).toBe(true);
  });
});

describe('screenshotSize', () => {
  it('given more table rows when sized then the picture is taller by whole rows', () => {
    const base = screenshotSize({ header: ['a'], tableRows: 1, noteLines: 1 });
    const more = screenshotSize({ header: ['a'], tableRows: 3, noteLines: 1 });

    expect(more.height - base.height).toBe(44);
    expect(more.width).toBe(GRAPH_SIZE.width + 48);
  });
});
