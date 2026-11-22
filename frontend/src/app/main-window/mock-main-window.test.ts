import { get } from 'svelte/store';
import { describe, expect, it } from 'vitest';
import { createMockMainWindow } from './mock-main-window';

describe('createMockMainWindow', () => {
  it('given the default mock, when read, then it is running with the Run chip and live traces', () => {
    const model = get(createMockMainWindow());

    expect(model.toolbar.running).toBe(true);
    expect(model.toolbar.chips[0]).toMatchObject({ kind: 'run', text: 'Run' });
    expect(model.graph.state).toBe('live');
  });

  it('given a running mock, when run is toggled, then it stops and the graph is frozen', () => {
    const mock = createMockMainWindow();

    mock.toggleRun();

    const model = get(mock);
    expect(model.toolbar.running).toBe(false);
    expect(model.toolbar.chips.map((chip) => chip.kind)).toEqual(['stop']);
    expect(model.graph.state).toBe('frozen');
  });

  it('given the mock, when read, then there are four channels and every trace value lies within the 8 divisions', () => {
    const model = get(createMockMainWindow());

    const values = model.graph.channels.flatMap((c) => c.samples);
    expect(model.channels).toHaveLength(4);
    expect(Math.min(...values)).toBeGreaterThanOrEqual(0);
    expect(Math.max(...values)).toBeLessThanOrEqual(8);
  });
});
