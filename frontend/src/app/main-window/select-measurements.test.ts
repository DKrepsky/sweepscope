import { describe, expect, it } from 'vitest';
import type { MeasurementsState } from '../../api';
import { DEFAULT_PINNED } from './pinned-measurements';
import { selectMeasurements } from './select-measurements';

type Measurement = MeasurementsState['channels'][number]['measurements'][number];
type MeasurementId = Measurement['id'];

const IDS: [MeasurementId, string][] = [
  ['frequency', 'Frequency'],
  ['period', 'Period'],
  ['vpp', 'Vpp'],
  ['vmax', 'Vmax'],
  ['vmin', 'Vmin'],
  ['vmean', 'Vmean'],
  ['vrms', 'Vrms'],
  ['positiveWidth', '+Width'],
  ['negativeWidth', '-Width'],
  ['dutyCycle', 'Duty'],
  ['riseTime', 'Rise time'],
  ['fallTime', 'Fall time'],
];

const valid = (id: MeasurementId, label: string): Measurement => ({
  id,
  label,
  value: { value: 1.91, text: '1.91', unit: 'V' },
  invalid: null,
});

const edgeTooFast = (text: string): Measurement => ({
  id: 'riseTime',
  label: 'Rise time',
  value: null,
  invalid: { reason: 'edgeTooFast', text },
});

function channel(number: number, overrides: Partial<Record<MeasurementId, Measurement>> = {}, unverified?: string) {
  return {
    channel: number,
    clipped: false,
    unverified: unverified ?? null,
    measurements: IDS.map(([id, label]) => overrides[id] ?? valid(id, label)),
  };
}

const slice = (...channels: ReturnType<typeof channel>[]): MeasurementsState => ({
  revision: 1,
  channels,
  accuracyNote: 'Volts accurate to 2 to 3 counts.',
});

describe('selectMeasurements', () => {
  it('given a valid cell, when selected, then the row shows the value and unit as valid', () => {
    const model = selectMeasurements(slice(channel(1)), DEFAULT_PINNED, 'live');

    expect(model.rows.find((row) => row.id === 'vpp')?.cells['ch1']).toEqual({
      state: 'valid',
      text: '1.91',
      unit: 'V',
    });
  });

  it('given an invalid cell, when selected, then it is invalid with a numbered footnote carrying the backend text', () => {
    const rise = edgeTooFast('Edge too fast for this sample rate');
    const model = selectMeasurements(slice(channel(1, { riseTime: rise })), DEFAULT_PINNED, 'live');

    expect(model.rows.find((row) => row.id === 'riseTime')?.cells['ch1']).toEqual({
      state: 'invalid',
      text: '-',
      footnote: 1,
    });
    expect(model.footnotes).toEqual([{ key: 1, text: 'Edge too fast for this sample rate' }]);
  });

  it('given the same reason on two channels, when selected, then both cells share one footnote', () => {
    const rise = edgeTooFast('Edge too fast');
    const model = selectMeasurements(
      slice(channel(1, { riseTime: rise }), channel(2, { riseTime: rise })),
      DEFAULT_PINNED,
      'live',
    );

    expect(model.footnotes).toHaveLength(1);
  });

  it('given an unverified channel, when selected, then its valid cells are unverified with the reason as tooltip', () => {
    const model = selectMeasurements(slice(channel(1, {}, 'Outside the verified range')), DEFAULT_PINNED, 'live');

    expect(model.rows[0]?.cells['ch1']).toMatchObject({ state: 'unverified', tooltip: 'Outside the verified range' });
  });

  it('given a pinned subset, when selected, then only those rows show and the pin menu lists all twelve', () => {
    const model = selectMeasurements(slice(channel(1)), ['vpp', 'period'], 'live');

    expect(model.rows.map((row) => row.id)).toEqual(['period', 'vpp']);
    expect(model.pinnable).toHaveLength(12);
    expect(model.pinnable.filter((entry) => entry.pinned).map((entry) => entry.id)).toEqual(['period', 'vpp']);
  });

  it('given a frozen graph, when selected, then the frozen label is Frozen', () => {
    expect(selectMeasurements(slice(channel(1)), DEFAULT_PINNED, 'frozen').frozenLabel).toBe('Frozen');
  });

  it('given a live graph, when selected, then there is no frozen label', () => {
    expect(selectMeasurements(slice(channel(1)), DEFAULT_PINNED, 'live').frozenLabel).toBeUndefined();
  });

  it('given two channels, when selected, then there is one coloured column per channel and the accuracy note', () => {
    const model = selectMeasurements(slice(channel(1), channel(3)), DEFAULT_PINNED, 'live');

    expect(model.columns).toEqual([
      { id: 'ch1', label: 'CH1', color: 'color-ch1' },
      { id: 'ch3', label: 'CH3', color: 'color-ch3' },
    ]);
    expect(model.accuracyNote).toBe('Volts accurate to 2 to 3 counts.');
  });

  it('given no slice, when selected, then there are no columns', () => {
    expect(selectMeasurements(undefined, DEFAULT_PINNED, 'live').columns).toEqual([]);
  });

  it('given an empty graph, when selected, then there are no columns', () => {
    expect(selectMeasurements(slice(channel(1)), DEFAULT_PINNED, 'empty').columns).toEqual([]);
  });
});
