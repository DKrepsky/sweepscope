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

    const values = model.graph.channels.flatMap((c) => Array.from(c.samples));
    expect(model.channels).toHaveLength(4);
    expect(Math.min(...values)).toBeGreaterThanOrEqual(0);
    expect(Math.max(...values)).toBeLessThanOrEqual(8);
  });

  it('given the mock, when read, then every stepped setting reports step availability and the connection is not simulated', () => {
    const model = get(createMockMainWindow());

    expect(model.horizontal.timePerDiv.step.faster.enabled).toBe(true);
    expect(model.channels.every((channel) => channel.voltsPerDiv.step.slower.enabled)).toBe(true);
    expect(model.status.connection.simulated).toBe(false);
    expect(model.banners).toEqual([]);
  });

  it('given channel 4 off, when it is enabled, then only that channel changes', () => {
    const mock = createMockMainWindow();

    mock.setChannelEnabled(4, true);

    expect(get(mock).channels.map((channel) => channel.enabled)).toEqual([true, true, true, true]);
  });

  it('given channel 2, when it is selected, then it is the only selected channel and the graph follows', () => {
    const mock = createMockMainWindow();

    mock.selectChannel(2);

    const model = get(mock);
    expect(model.channels.filter((channel) => channel.selected).map((channel) => channel.number)).toEqual([2]);
    expect(model.graph.selected).toBe(2);
  });

  it('given channel 1 at probe 1x, when the probe is cycled three times, then it is back at 1x', () => {
    const mock = createMockMainWindow();

    mock.cycleProbe(1);
    expect(get(mock).channels[0]?.probe).toBe('10×');
    mock.cycleProbe(1);
    mock.cycleProbe(1);

    expect(get(mock).channels[0]?.probe).toBe('1×');
  });

  it('given the mock, when coupling, bandwidth, invert and trigger choices are made, then the model shows them', () => {
    const mock = createMockMainWindow();

    mock.setCoupling(1, 'ac');
    mock.toggleBandwidth(1);
    mock.toggleInvert(1);
    mock.setTriggerSource(3);
    mock.setTriggerSlope('falling');
    mock.setTriggerSweep('normal');
    mock.setInterpolation('dots');

    const model = get(mock);
    expect(model.channels[0]).toMatchObject({ coupling: 'ac', bandwidthLimit: true, inverted: true });
    expect(model.trigger).toMatchObject({ source: 3, slope: 'falling', sweep: 'normal' });
    expect(model.horizontal.interpolation).toBe('dots');
  });

  it('given the mock, when the view, XY axis, persistence and peak mode are chosen, then the model shows them', () => {
    const mock = createMockMainWindow();

    mock.setView('xy');
    mock.setXyAxis('y', 3);
    mock.setPersistence('2s');
    mock.setPeakMode('manual');

    const model = get(mock);
    expect(model.views.tab).toBe('xy');
    expect(model.views.xy.y).toBe(3);
    expect(model.persistence.value).toBe('2s');
    expect(model.peak.mode).toBe('manual');
  });

  it('given the mock, when persistence is cleared and math switched on, then the epoch advances and the row is enabled', () => {
    const mock = createMockMainWindow();
    const before = get(mock).persistence.epoch;

    mock.clearPersistence();
    mock.setMathEnabled(true);

    const model = get(mock);
    expect(model.persistence.epoch).toBe(before + 1);
    expect(model.math.row.enabled).toBe(true);
  });
});
