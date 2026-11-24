import { describe, expect, it } from 'vitest';
import { tokens } from '../../design-system';
import { Canvas2dRenderer } from './canvas-2d-renderer';
import type { GraphFrame, PixelSize } from './graph-frame';
import { createRecordingContext } from './recording-context';

const size: PixelSize = { width: 1000, height: 800, ratio: 1 };

function render(frame: GraphFrame) {
  const graticule = createRecordingContext();
  const traces = createRecordingContext();
  new Canvas2dRenderer(graticule.ctx, traces.ctx).draw(frame, size);
  return { graticule, traces };
}

describe('XY renderer', () => {
  const xy = { x: [4, 0, 8], y: [0, 4, 8], xChannel: 1, yChannel: 3 } as const;
  const frame: GraphFrame = { state: 'live', channels: [], xy };

  it('given X and Y samples, when drawn, then the first and last points land at 0.5 + (4 - X) / 10 and Y / 8 of the screen', () => {
    const { points } = render(frame).traces.strokes[0]!;
    expect(points[0]).toEqual([0.5 * size.width, 0]);
    expect(points[2]![0]).toBeCloseTo(0.1 * size.width, 9);
    expect(points[2]![1]).toBeCloseTo(size.height, 9);
  });

  it('given divisions of 100 by 100 px, when X and Y both change by one division, then the figure moves equally in both directions', () => {
    const { points } = render({ ...frame, xy: { ...xy, x: [4, 3], y: [4, 5] } }).traces.strokes[0]!;
    const dx = points[1]![0] - points[0]![0];
    const dy = points[1]![1] - points[0]![1];
    expect([dx, dy]).toEqual([size.width / 10, size.height / 8]);
  });

  it('given an XY frame, when drawn, then one trace is drawn in the Y channel colour with straight segments', () => {
    const { strokes } = render(frame).traces;
    expect(strokes).toHaveLength(1);
    expect(strokes[0]!.style).toBe(tokens['color-ch3']);
    expect(strokes[0]!.points).toHaveLength(3);
  });

  it('given XY samples beyond the screen, when drawn, then the points are held on the screen edge', () => {
    const { points } = render({ ...frame, xy: { ...xy, x: [-40], y: [40] } }).traces.strokes[0]!;
    expect(points[0]).toEqual([size.width, size.height]);
  });

  it('given an XY frame with a NaN sample, when drawn, then the lone point before it is a dot and the path restarts after it', () => {
    const { points } = render({ ...frame, xy: { ...xy, x: [4, NaN, 4, 3], y: [1, 1, 2, 2] } }).traces.strokes[0]!;
    expect(points).toHaveLength(4);
    expect(points[0]).toEqual(points[1]);
  });

  it('given an XY frame and dots mode, when drawn, then segments are still drawn, not dots', () => {
    const { traces } = render({ ...frame, dots: true });
    expect(traces.strokes).toHaveLength(1);
    expect(traces.fills).toEqual([]);
  });

  it('given an XY frame, when drawn, then the channel traces of the frame are not drawn', () => {
    const { strokes } = render({ ...frame, channels: [{ channel: 1, samples: [1, 2] }] }).traces;
    expect(strokes).toHaveLength(1);
  });
});

describe('NaN line breaks', () => {
  const drawn = (samples: number[]) =>
    render({ state: 'live', channels: [{ channel: 1, samples }] }).traces.strokes[0]!.points;

  it('given a NaN sample between two runs, when drawn, then no point is placed for it and the path restarts after it', () => {
    const points = drawn([1, 2, NaN, 3, 4]);
    expect(points).toHaveLength(4);
    expect(points.every(([x, y]) => !Number.isNaN(x) && !Number.isNaN(y))).toBe(true);
  });

  it('given a lone sample between NaN samples, when drawn, then it is a zero-length segment so the round cap shows a dot', () => {
    const points = drawn([NaN, 2, NaN]);
    expect(points).toHaveLength(2);
    expect(points[0]).toEqual(points[1]);
  });

  it('given all samples NaN, when drawn, then nothing is placed', () => {
    expect(drawn([NaN, NaN])).toEqual([]);
  });

  it('given dots mode with a NaN sample, when drawn, then only the real samples get a dot', () => {
    const { traces } = render({ state: 'live', dots: true, channels: [{ channel: 1, samples: [1, NaN, 3] }] });
    expect(traces.fills).toHaveLength(2);
  });
});

describe('tick labels', () => {
  const frame: GraphFrame = {
    state: 'live',
    channels: [],
    ticksX: [
      { at: 0, text: '0 s' },
      { at: 0.5, text: '1 ms' },
      { at: 1, text: '2 ms' },
    ],
    ticksY: [
      { at: 0, text: '-4 V' },
      { at: 0.5, text: '0 V' },
      { at: 1, text: '4 V' },
    ],
  };

  it('given ticks on both axes, when drawn, then every label is on the graticule layer in the muted text colour', () => {
    const { graticule } = render(frame);
    expect(graticule.texts.map((t) => t.text)).toEqual(['0 s', '1 ms', '2 ms', '-4 V', '0 V', '4 V']);
    expect(graticule.texts.every((t) => t.style === tokens['color-text-muted'])).toBe(true);
  });

  it('given ticks, when drawn, then every label anchor is inside the frame', () => {
    for (const { x, y } of render(frame).graticule.texts) {
      expect(x).toBeGreaterThanOrEqual(0);
      expect(x).toBeLessThanOrEqual(size.width);
      expect(y).toBeGreaterThan(0);
      expect(y).toBeLessThan(size.height);
    }
  });

  it('given labels at the screen edges, when drawn, then they are anchored inward so the text stays inside', () => {
    const texts = render(frame).graticule.texts;
    const byText = (text: string) => texts.find((t) => t.text === text)!;
    expect(byText('0 s').align).toBe('left');
    expect(byText('2 ms').align).toBe('right');
    expect(byText('1 ms').align).toBe('center');
    expect(byText('-4 V').baseline).toBe('bottom');
    expect(byText('4 V').baseline).toBe('top');
  });

  it('given the same size and new tick text, when drawn again, then the graticule is redrawn with the new labels', () => {
    const graticule = createRecordingContext();
    const renderer = new Canvas2dRenderer(graticule.ctx, createRecordingContext().ctx);
    renderer.draw(frame, size);
    renderer.draw({ ...frame, ticksX: [{ at: 0.5, text: '5 ms' }], ticksY: [] }, size);
    expect(graticule.texts.at(-1)?.text).toBe('5 ms');
  });

  it('given unchanged ticks, when drawn again, then the graticule is not redrawn', () => {
    const graticule = createRecordingContext();
    const renderer = new Canvas2dRenderer(graticule.ctx, createRecordingContext().ctx);
    renderer.draw(frame, size);
    renderer.draw({ ...frame, ticksX: [...frame.ticksX!] }, size);
    expect(graticule.texts).toHaveLength(6);
  });
});
