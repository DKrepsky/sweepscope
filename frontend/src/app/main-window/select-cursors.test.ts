import { describe, expect, it } from 'vitest';
import type { InstrumentState } from '../../api';
import { instrumentState } from '../instrument/test-data';
import { selectCursors } from './select-cursors';

const scale = (value: number) => ({
  index: 0,
  label: { value, text: String(value), unit: '' },
  faster: { enabled: true, reason: null },
  slower: { enabled: true, reason: null },
});

function instrument(secondsPerDiv: number, voltsPerDiv: number, positionDivs: number, enabled = true): InstrumentState {
  const state = instrumentState();
  return {
    ...state,
    channels: state.channels.map((channel) =>
      channel.number === 1 ? { ...channel, enabled, range: scale(voltsPerDiv), positionDivs } : channel,
    ),
    horizontal: { ...state.horizontal, timeBase: scale(secondsPerDiv) },
  };
}

const valueOf = (model: ReturnType<typeof selectCursors>, label: string) =>
  model.values.find((entry) => entry.label === label)?.value;

describe('cursor readouts', () => {
  it('given X1 and X2 at 2 ms/div, when selected, then X readouts count from the left edge over ten divisions', () => {
    const model = selectCursors({ x1: 0.035, x2: 0.055 }, instrument(2e-3, 0.5, 0), 'live', 1);
    expect(valueOf(model, 'X1')).toBe('700 us');
    expect(valueOf(model, 'X2')).toBe('1.10 ms');
    expect(valueOf(model, 'ΔX')).toBe('400 us');
    expect(valueOf(model, '1/ΔX')).toBe('2.50 kHz');
  });

  it('given Y1 and Y2 on 500 mV/div at position -2 div, when selected, then volts follow the divisions minus the position', () => {
    const model = selectCursors({ y1: 0.625, y2: 0.375 }, instrument(2e-3, 0.5, -2), 'live', 1);
    expect(valueOf(model, 'Y1')).toBe('500 mV');
    expect(valueOf(model, 'Y2')).toBe('1.50 V');
    expect(valueOf(model, 'ΔY')).toBe('1.00 V');
  });

  it('given the scale changes, when selected again, then readouts change and the lines stay', () => {
    const cursors = { x1: 0.2, y1: 0.25 };
    const before = selectCursors(cursors, instrument(1e-3, 1, 0), 'live', 1);
    const after = selectCursors(cursors, instrument(2e-3, 2, 0), 'live', 1);
    expect(valueOf(after, 'X1')).not.toBe(valueOf(before, 'X1'));
    expect(valueOf(after, 'Y1')).not.toBe(valueOf(before, 'Y1'));
    expect(after.lines.map((line) => line.position)).toEqual(before.lines.map((line) => line.position));
  });

  it('given X1 equal to X2, when selected, then 1/ΔX is not measured', () => {
    const model = selectCursors({ x1: 0.4, x2: 0.4 }, instrument(1e-3, 1, 0), 'live', 1);
    expect(valueOf(model, '1/ΔX')).toBe('-');
  });

  it('given the selected channel is off, when selected, then the Y readouts are not measured', () => {
    const model = selectCursors({ y1: 0.3, y2: 0.7, x1: 0.1 }, instrument(1e-3, 1, 0, false), 'live', 1);
    expect(['Y1', 'Y2', 'ΔY'].map((label) => valueOf(model, label))).toEqual(['-', '-', '-']);
    expect(valueOf(model, 'X1')).not.toBe('-');
  });

  it('given a cursor is absent, when selected, then its readouts and the deltas are not measured', () => {
    const model = selectCursors({ x1: 0.1 }, instrument(1e-3, 1, 0), 'live', 1);
    expect(['X2', 'Y1', 'ΔX', '1/ΔX'].map((label) => valueOf(model, label))).toEqual(['-', '-', '-', '-']);
    expect(model.delta).toBeUndefined();
  });

  it('given no instrument state, when selected, then nothing is measured', () => {
    const model = selectCursors({ x1: 0.1, y1: 0.2 }, undefined, 'live', 1);
    expect([valueOf(model, 'X1'), valueOf(model, 'Y1')]).toEqual(['-', '-']);
  });
});

describe('cursor lines and availability', () => {
  it('given no cursors, when selected, then the model is off and has no lines', () => {
    const model = selectCursors({}, instrument(1e-3, 1, 0), 'live', 2);
    expect(model).toMatchObject({ enabled: false, source: 2, lines: [] });
  });

  it('given all four cursors, when selected, then lines carry axis, position and a tag with the value', () => {
    const model = selectCursors({ x1: 0.1, x2: 0.3, y1: 0.5, y2: 0.25 }, instrument(1e-3, 1, 0), 'live', 1);
    expect(model.lines.map((line) => [line.id, line.axis, line.position])).toEqual([
      ['x1', 'x', 0.1],
      ['x2', 'x', 0.3],
      ['y1', 'y', 0.5],
      ['y2', 'y', 0.25],
    ]);
    expect(model.lines[0]?.tag).toBe('X1 1.00 ms');
    expect(model.delta).toEqual({ tag: 'ΔX 2.00 ms', position: 0.2 });
  });

  it('given graph state empty, when selected, then cursors are unavailable with the reason No waveform', () => {
    const model = selectCursors({}, instrument(1e-3, 1, 0), 'empty', 1);
    expect(model.available).toEqual({ enabled: false, reason: 'No waveform' });
  });

  it('given graph state frozen, when selected, then cursors are available', () => {
    const model = selectCursors({}, instrument(1e-3, 1, 0), 'frozen', 1);
    expect(model.available).toEqual({ enabled: true });
  });
});
