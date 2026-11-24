import { describe, expect, it } from 'vitest';
import type { InstrumentFacts, InstrumentState, MeasurementsState } from '../../api';
import { selectChannels } from './select-channels';
import { selectMeasurements } from './select-measurements';
import { selectStatus } from './select-status';

type Measurement = MeasurementsState['channels'][number]['measurements'][number];
type Warning = MeasurementsState['warnings'][number];

const MESSAGE =
  'Possible aliasing on CH1. Its frequency (62.0 MHz) is more than one fifth of the sample rate (250 MS/s), so the display may show a wrong shape or frequency.';

const warning = (channel: number, message = MESSAGE): Warning => ({
  id: `alias-ch${channel}`,
  severity: 'warning',
  channel,
  short: `Alias CH${channel}`,
  message,
});

const measure = (id: Measurement['id'], label: string, text: string, unit: string): Measurement => ({
  id,
  label,
  value: { value: 1, text, unit },
  invalid: null,
});

const channel = (number: number) => ({
  channel: number,
  clipped: false,
  unverified: null,
  measurements: [
    measure('frequency', 'Frequency', '62.0', 'MHz'),
    measure('period', 'Period', '16.1', 'ns'),
    measure('vpp', 'Vpp', '1.91', 'mV'),
  ],
});

const slice = (channels: number[], warnings: Warning[] = []): MeasurementsState => ({
  revision: 1,
  channels: channels.map(channel),
  accuracyNote: '',
  warnings,
});

const PINNED = ['frequency', 'period', 'vpp'];

describe('alias in the measurement matrix', () => {
  it('given a warning on channel 1, when selected, then only its column carries the badge', () => {
    const { columns } = selectMeasurements(slice([1, 2], [warning(1)]), PINNED, 'live');

    expect(columns.map((column) => column.alias?.short)).toEqual(['Alias CH1', undefined]);
  });

  it('given a warning on channel 1, when selected, then its Frequency and Period cells carry the footnote and the edge', () => {
    const model = selectMeasurements(slice([1], [warning(1)]), PINNED, 'live');

    const cells = model.rows.map((row) => row.cells['ch1']);
    expect(cells.map((cell) => cell?.warning)).toEqual([true, true, undefined]);
    expect(model.footnotes).toEqual([{ key: 1, text: MESSAGE }]);
  });

  it('given the warning clears, when selected, then no badge, edge or footnote remains', () => {
    const model = selectMeasurements(slice([1]), PINNED, 'live');

    expect(model.columns[0]?.alias).toBeUndefined();
    expect(model.footnotes).toEqual([]);
    expect(model.rows.some((row) => row.cells['ch1']?.warning)).toBe(false);
  });
});

describe('math column', () => {
  it('given channel 5, when selected, then the column is the white M', () => {
    const { columns } = selectMeasurements(slice([1, 5]), PINNED, 'live');

    expect(columns[1]).toEqual({ id: 'ch5', label: 'M', color: 'color-math' });
  });

  it('given a product of two channels, when selected, then voltage cells of M read in volts squared with the prefix', () => {
    const model = selectMeasurements(slice([1, 5]), PINNED, 'live', { mathUnit: 'V²' });

    const units = model.rows.map((row) => row.cells['ch5']?.unit);
    expect(units).toEqual(['MHz', 'ns', 'mV²']);
  });

  it('given a product, when selected, then the channel columns keep volts', () => {
    const model = selectMeasurements(slice([1, 5]), PINNED, 'live', { mathUnit: 'V²' });

    expect(model.rows.find((row) => row.id === 'vpp')?.cells['ch1']?.unit).toBe('mV');
  });

  it('given a difference, when selected, then M keeps the unit of the backend', () => {
    const model = selectMeasurements(slice([5]), PINNED, 'live', { mathUnit: 'V' });

    expect(model.rows.find((row) => row.id === 'vpp')?.cells['ch5']?.unit).toBe('mV');
  });
});

describe('roll mode', () => {
  it('given roll, when selected, then no column, row or value shows and the reason is given', () => {
    const model = selectMeasurements(slice([1]), PINNED, 'live', { roll: true });

    expect(model.columns).toEqual([]);
    expect(model.rows).toEqual([]);
    expect(model.unavailableNote).toMatch(/roll mode/);
  });
});

const instrument = {
  controls: { vertical: { enabled: true } },
  channels: [1, 2].map((number) => ({
    number,
    enabled: true,
    range: { label: { text: '500', unit: 'mV' }, faster: { enabled: true }, slower: { enabled: true } },
    position: { text: '0.00', unit: 'div' },
    positionDivs: 0,
    coupling: 'dc',
    bandwidthLimit: false,
    probeText: '1x',
    invert: false,
    unverified: null,
    enable: { enabled: true },
  })),
} as unknown as InstrumentState;

describe('alias and roles on the scale strip model', () => {
  it('given a warning on channel 2, when selected, then only channel 2 has the alias notice', () => {
    const channels = selectChannels(undefined, instrument, 1, slice([1, 2], [warning(2)]));

    expect(channels.map((model) => model.alias?.short)).toEqual([undefined, 'Alias CH2']);
  });

  it('given the XY view, when selected, then the axes carry the role letters', () => {
    const channels = selectChannels(undefined, instrument, 1, undefined, { x: 2, y: 1 });

    expect(channels.map((model) => model.role)).toEqual(['y', 'x']);
  });

  it('given the Yt view, when selected, then no channel has a role', () => {
    const channels = selectChannels(undefined, instrument, 1);

    expect(channels.some((model) => model.role)).toBe(false);
  });
});

describe('alias in the status bar model', () => {
  const facts = { sampleRate: { value: 250e6, text: '250', unit: 'MS/s' } } as unknown as InstrumentFacts;

  it('given a warning, when selected, then the sample rate carries its short text and message', () => {
    const status = selectStatus(undefined, undefined, facts, 'gpu', slice([1], [warning(1)]));

    expect(status.sampleRate.alias).toEqual({ short: 'Alias CH1', message: MESSAGE });
  });

  it('given warnings on two channels, when selected, then the count is shown and the messages are joined', () => {
    const status = selectStatus(
      undefined,
      undefined,
      facts,
      'gpu',
      slice([1, 2], [warning(1, 'a.'), warning(2, 'b.')]),
    );

    expect(status.sampleRate.alias).toEqual({ short: 'Alias CH1 +1', message: 'a. b.' });
  });

  it('given no warning, when selected, then the sample rate has no alias', () => {
    const status = selectStatus(undefined, undefined, facts, 'gpu', slice([1]));

    expect(status.sampleRate.alias).toBeUndefined();
  });
});
