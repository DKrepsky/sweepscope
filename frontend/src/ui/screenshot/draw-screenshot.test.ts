import { describe, expect, it } from 'vitest';
import { tokens } from '../../design-system';
import { createRecordingContext } from '../graph/recording-context';
import { drawScreenshot, type ScreenshotSpec } from './draw-screenshot';
import { GRAPH_SIZE, screenshotLayout, screenshotSize } from './screenshot-layout';

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

    const layout = screenshotLayout({ header: spec().header, tableRows: 1, footnotes: 1 });
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

describe('screenshotSize', () => {
  it('given more table rows when sized then the picture is taller by whole rows', () => {
    const base = screenshotSize({ header: ['a'], tableRows: 1, footnotes: 0 });
    const more = screenshotSize({ header: ['a'], tableRows: 3, footnotes: 0 });

    expect(more.height - base.height).toBe(44);
    expect(more.width).toBe(GRAPH_SIZE.width + 48);
  });
});
