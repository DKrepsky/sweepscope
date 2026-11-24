import { describe, expect, it } from 'vitest';
import { decodeRollItem, type RollItem } from './decode-roll';
import { createRollWindow } from './roll-window';
import { buildRollItem, type TestRollItem } from './test-roll';

const item = (options: TestRollItem): RollItem => decodeRollItem(buildRollItem(options));
const ramp = (from: number, count: number) => Array.from({ length: count }, (_, i) => from + i);
const whole = (samples: Float32Array) => Array.from(samples, (v) => Math.round(v));

describe('roll window', () => {
  it('given pushes totalling more than the window, when viewed, then it holds exactly the window with the newest at the right', () => {
    const window = createRollWindow(10);
    window.push(item({ firstIndex: 0, samples: ramp(0, 8) }));
    window.push(item({ firstIndex: 8, samples: ramp(8, 8) }));
    const samples = window.view().channels[0]?.samples;
    expect(whole(samples ?? new Float32Array())).toEqual(ramp(6, 10));
  });

  it('given fewer samples than the window, when viewed, then the left part is NaN and the newest sit at the right', () => {
    const window = createRollWindow(5);
    window.push(item({ samples: [1, 2] }));
    const samples = whole(window.view().channels[0]?.samples ?? new Float32Array());
    expect(samples).toEqual([NaN, NaN, NaN, 1, 2]);
  });

  it('given one push larger than the window, when viewed, then it keeps the newest samples', () => {
    const window = createRollWindow(4);
    window.push(item({ samples: ramp(0, 9) }));
    expect(whole(window.view().channels[0]?.samples ?? new Float32Array())).toEqual([5, 6, 7, 8]);
  });

  it('given a gap item, when viewed, then the covered samples are NaN and one gap entry spans them', () => {
    const window = createRollWindow(10);
    window.push(item({ firstIndex: 0, samples: ramp(0, 4) }));
    window.push(item({ kind: 'gap', firstIndex: 3, missed: 2 }));
    window.push(item({ firstIndex: 6, samples: [6, 7] }));
    const view = window.view();
    expect(whole(view.channels[0]?.samples ?? new Float32Array())).toEqual([NaN, NaN, 0, 1, 2, 3, NaN, NaN, 6, 7]);
    expect(view.gaps).toEqual([{ from: 0.6, to: 0.8, missed: 2, approximate: true }]);
    expect(view.gapCount).toBe(1);
  });

  it('given samples that skip indices without a gap item, when viewed, then the missing run is treated as a gap', () => {
    const window = createRollWindow(10);
    window.push(item({ firstIndex: 0, samples: ramp(0, 2) }));
    window.push(item({ firstIndex: 5, samples: [5] }));
    const view = window.view();
    expect(whole(view.channels[0]?.samples ?? new Float32Array()).slice(-6)).toEqual([0, 1, NaN, NaN, NaN, 5]);
    expect(view.gaps).toHaveLength(1);
  });

  it('given a gap, when enough samples arrive to push it out of the window, then the entry is gone and the count stays', () => {
    const window = createRollWindow(4);
    window.push(item({ firstIndex: 0, samples: [0] }));
    window.push(item({ kind: 'gap', firstIndex: 0, missed: 1 }));
    window.push(item({ firstIndex: 2, samples: ramp(2, 3) }));
    expect(window.view().gaps).toHaveLength(1);
    window.push(item({ firstIndex: 5, samples: [5] }));
    const view = window.view();
    expect(view.gaps).toEqual([]);
    expect(view.gapCount).toBe(1);
  });

  it('given a gap partly outside the window, when viewed, then its fractions are clipped to the window', () => {
    const window = createRollWindow(4);
    window.push(item({ firstIndex: 0, samples: [0] }));
    window.push(item({ kind: 'gap', firstIndex: 0, missed: 3 }));
    window.push(item({ firstIndex: 4, samples: [4, 5] }));
    expect(window.view().gaps).toEqual([{ from: 0, to: 0.5, missed: 3, approximate: true }]);
  });

  it('given a new session, when its first item arrives, then the window is cleared', () => {
    const window = createRollWindow(4);
    window.push(item({ session: 1, samples: [1, 2, 3, 4] }));
    window.push(item({ kind: 'gap', session: 1, firstIndex: 3, missed: 1 }));
    window.push(item({ session: 2, firstIndex: 0, samples: [9] }));
    const view = window.view();
    expect(whole(view.channels[0]?.samples ?? new Float32Array())).toEqual([NaN, NaN, NaN, 9]);
    expect(view.gaps).toEqual([]);
    expect(view.gapCount).toBe(0);
  });

  it('given a repeated item, when pushed again, then it is ignored', () => {
    const window = createRollWindow(4);
    window.push(item({ firstIndex: 0, samples: [1, 2] }));
    window.push(item({ firstIndex: 0, samples: [1, 2] }));
    expect(whole(window.view().channels[0]?.samples ?? new Float32Array())).toEqual([NaN, NaN, 1, 2]);
  });

  it('given a closed item, when pushed, then the samples stay', () => {
    const window = createRollWindow(3);
    window.push(item({ samples: [1, 2, 3] }));
    window.push(item({ kind: 'closed' }));
    expect(whole(window.view().channels[0]?.samples ?? new Float32Array())).toEqual([1, 2, 3]);
  });

  it('given CH1 and the math trace, when viewed, then each channel has its own samples in ascending order', () => {
    const window = createRollWindow(2);
    window.push(item({ channels: [5, 1], samples: [3, 4] }));
    expect(window.view().channels.map(({ channel }) => channel)).toEqual([1, 5]);
  });

  it('given a view, when the window changes afterwards, then the earlier view is unchanged', () => {
    const window = createRollWindow(2);
    window.push(item({ samples: [1, 2] }));
    const before = window.view();
    window.push(item({ firstIndex: 2, samples: [3] }));
    expect(whole(before.channels[0]?.samples ?? new Float32Array())).toEqual([1, 2]);
  });

  it('given 10 000 pushes, when viewed, then the window is still the fixed size', () => {
    const window = createRollWindow();
    for (let i = 0; i < 10_000; i++) window.push(item({ firstIndex: i * 2, samples: [1, 2] }));
    const view = window.view();
    expect(view.channels).toHaveLength(1);
    expect(view.channels[0]?.samples).toHaveLength(2500);
  });
});
