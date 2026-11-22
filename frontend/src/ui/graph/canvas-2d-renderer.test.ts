import { describe, expect, it } from 'vitest';
import { tokens } from '../../design-system';
import { Canvas2dRenderer } from './canvas-2d-renderer';
import type { GraphFrame, PixelSize } from './graph-frame';
import { createRecordingContext } from './recording-context';
import type { StrokeRecord } from './recording-context';

const size: PixelSize = { width: 500, height: 400, ratio: 1 };
const flat = (y: number) => Array.from({ length: 11 }, () => y);

function render(frame: GraphFrame, pixels: PixelSize = size) {
  const graticule = createRecordingContext();
  const traces = createRecordingContext();
  new Canvas2dRenderer(graticule.ctx, traces.ctx).draw(frame, pixels);
  return { graticule, traces };
}

const everything: GraphFrame = {
  state: 'live',
  channels: [1, 2, 3, 4].map((channel) => ({ channel: channel as 1 | 2 | 3 | 4, samples: flat(channel) })),
  reference: { samples: flat(2) },
  math: { samples: flat(3) },
};

describe('Canvas2dRenderer draw order', () => {
  it('AC-G1: given channels 1 to 4, a reference and a math trace, when drawn, then strokes go reference, math, CH4, CH3, CH2, CH1 in token colours', () => {
    const { traces } = render(everything);
    expect(traces.strokes.map((s) => s.style)).toEqual([
      tokens['color-ref'],
      tokens['color-math'],
      tokens['color-ch4'],
      tokens['color-ch3'],
      tokens['color-ch2'],
      tokens['color-ch1'],
    ]);
  });

  it('AC-G1: given channel 3 selected, when drawn, then it is drawn last, on top', () => {
    const { traces } = render({ ...everything, selected: 3 });
    expect(traces.strokes.map((s) => s.style).slice(-4)).toEqual([
      tokens['color-ch4'],
      tokens['color-ch2'],
      tokens['color-ch1'],
      tokens['color-ch3'],
    ]);
  });

  it('given the reference and math traces, when drawn, then reference is dashed 6/4 at 1 px and math solid at 1 px', () => {
    const { traces } = render(everything);
    const [ref, math] = traces.strokes as [StrokeRecord, StrokeRecord];
    expect([ref.dash, ref.width, math.dash, math.width]).toEqual([[6, 4], 1, [], 1]);
  });

  it('given a channel trace, when drawn, then its width is the trace-width token', () => {
    const { traces } = render(everything);
    expect(traces.strokes.at(-1)?.width).toBe(1.5);
  });
});

describe('Canvas2dRenderer states', () => {
  it('AC-G1: given state empty, when drawn, then only the graticule is drawn', () => {
    const { graticule, traces } = render({ ...everything, state: 'empty' });
    expect(traces.strokes).toEqual([]);
    expect(graticule.strokes.length).toBeGreaterThan(0);
  });

  it('AC-G1: given state frozen, when drawn, then traces use opacity 0.4 and the graticule is unchanged', () => {
    const frozen = render({ ...everything, state: 'frozen' });
    const live = render(everything);
    expect(frozen.traces.strokes.every((s) => s.alpha === 0.4)).toBe(true);
    expect(live.traces.strokes.every((s) => s.alpha === 1)).toBe(true);
    expect(frozen.graticule.strokes).toEqual(live.graticule.strokes);
  });

  it('given dots mode, when drawn, then each sample is a 3 px square and no line connects them', () => {
    const { traces } = render({ state: 'live', dots: true, channels: [{ channel: 1, samples: [1, 2, 3] }] });
    expect(traces.strokes).toEqual([]);
    expect(traces.fills).toHaveLength(3);
    expect(traces.fills[0]!.style).toBe(tokens['color-ch1']);
  });

  it('given the same size drawn twice, when drawn, then the graticule is not redrawn', () => {
    const graticule = createRecordingContext();
    const renderer = new Canvas2dRenderer(graticule.ctx, createRecordingContext().ctx);
    renderer.draw(everything, size);
    const count = graticule.strokes.length;
    renderer.draw(everything, size);
    expect(graticule.strokes).toHaveLength(count);
  });
});

describe('Canvas2dRenderer clipping', () => {
  const above: GraphFrame = { state: 'live', channels: [{ channel: 2, samples: [-3, -3, -3, 4, 4] }] };

  it('AC-G1: given samples above the top border, when drawn, then they are clamped to the border and a channel-coloured triangle is added', () => {
    const { traces } = render(above);
    const edge = traces.strokes.find((s) => s.width === 2);
    expect(edge?.points.every(([, y]) => y === 0)).toBe(true);
    expect(edge?.style).toBe(tokens['color-ch2']);
    expect(traces.fills.filter((f) => f.kind === 'path' && f.style === tokens['color-ch2'])).toHaveLength(1);
    expect(traces.strokes[0]!.points.every(([, y]) => y >= 0)).toBe(true);
  });

  it('given samples below the bottom border, when drawn, then the triangle points down off the screen', () => {
    const { traces } = render({ state: 'live', channels: [{ channel: 1, samples: [9, 9, 9] }] });
    const tip = traces.fills[0]!.points[0]!;
    expect(tip[1]).toBe(size.height);
  });
});

describe('Canvas2dRenderer HiDPI crispness', () => {
  it.each([1, 1.25, 1.5, 2])(
    'AC-G2: given ratio %s, when the graticule is drawn, then lines sit on device pixel centres and are at least one device pixel wide',
    (ratio) => {
      const { graticule } = render(everything, { width: 523, height: 418, ratio });
      const major = graticule.strokes.filter((s) => s.style === tokens['color-grid-major']);
      expect(major.length).toBeGreaterThan(0);
      for (const stroke of graticule.strokes) expect(stroke.width * ratio).toBeGreaterThanOrEqual(1 - 1e-9);
      for (const [a, b] of pairs(major[0]!.points)) {
        const fixed = a[0] === b[0] ? a[0] : a[1];
        const device = fixed * ratio;
        expect(Math.abs(device - Math.floor(device) - 0.5)).toBeLessThan(1e-6);
      }
    },
  );
});

function pairs(points: [number, number][]) {
  const out: [[number, number], [number, number]][] = [];
  for (let i = 0; i < points.length; i += 2) out.push([points[i]!, points[i + 1]!]);
  return out;
}
