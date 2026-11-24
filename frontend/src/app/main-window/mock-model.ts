import type { ChannelModel, MainWindowModel, MeasurementsModel, PersistenceModel, ScaleQuantity } from './types';
import { sineWave, squareWave, triangleWave } from './mock-signals';

/** Illustrative placeholders from the design prototype; not hardware statements. */

export const scale = (value: string, unit: string): ScaleQuantity => ({
  value,
  unit,
  step: { faster: { enabled: true }, slower: { enabled: true } },
});

const channel = (overrides: Partial<ChannelModel> & Pick<ChannelModel, 'number'>): ChannelModel => ({
  enabled: true,
  selected: false,
  voltsPerDiv: scale('1', 'V/div'),
  position: { value: '0.00', unit: 'div' },
  coupling: 'dc',
  bandwidthLimit: false,
  probe: '1×',
  inverted: false,
  groundPosition: 0.5,
  ...overrides,
});

export const mockChannels = (): ChannelModel[] => [
  channel({
    number: 1,
    selected: true,
    voltsPerDiv: scale('500', 'mV/div'),
    position: { value: '−2.00', unit: 'div' },
    groundPosition: 0.25,
  }),
  channel({
    number: 2,
    voltsPerDiv: scale('20', 'mV/div'),
    position: { value: '+3.00', unit: 'div' },
    groundPosition: 0.875,
    unverified: 'Unverified range: below 50 mV/div. Accuracy up to about 5 % of a division.',
  }),
  channel({
    number: 3,
    voltsPerDiv: scale('2', 'V/div'),
    unverified: 'Unverified range: above 1 V/div.',
  }),
  channel({
    number: 4,
    enabled: false,
    enableReason: 'Channel 4 off. Enabling it keeps 250 MS/s (3 or 4 channels).',
  }),
];

const valid = (text: string, unit?: string) => ({ state: 'valid' as const, text, unit });
const unverified = (text: string, unit: string) => ({ state: 'unverified' as const, text, unit });
const unavailable = (footnote: number) => ({ state: 'invalid' as const, text: '', footnote });

function measurementRows(): MeasurementsModel['rows'] {
  const row = (
    id: string,
    label: string,
    ch1: MeasurementsModel['rows'][number]['cells'][string],
    ch2 = ch1,
    ch3 = ch1,
  ) => ({
    id,
    label,
    cells: { ch1, ch2, ch3 },
  });
  return [
    row('frequency', 'Frequency', valid('1.000', 'kHz'), valid('2.000', 'kHz'), unavailable(2)),
    row('period', 'Period', valid('1.000', 'ms'), valid('500.0', 'µs'), unavailable(2)),
    row('vpp', 'Vpp', valid('1.91', 'V'), unverified('32.0', 'mV'), unverified('4.02', 'V')),
    row('vmax', 'Vmax', valid('1.93', 'V'), unverified('16.0', 'mV'), unverified('2.01', 'V')),
    row('vmin', 'Vmin', valid('20.0', 'mV'), unverified('−16.0', 'mV'), unverified('−2.01', 'V')),
    row('vmean', 'Vmean', valid('960', 'mV'), unverified('−0.3', 'mV'), unverified('−10', 'mV')),
    row('vrms', 'Vrms', valid('1.35', 'V'), unverified('11.3', 'mV'), unverified('1.15', 'V')),
    row('width', '+Width', valid('500.0', 'µs'), valid('250.0', 'µs'), unavailable(2)),
    row('duty', 'Duty', valid('50.0', '%'), valid('50.0', '%'), unavailable(2)),
    row('rise', 'Rise time', unavailable(1), unavailable(3), unavailable(3)),
    row('fall', 'Fall time', unavailable(1), unavailable(3), unavailable(3)),
  ];
}

export const mockMeasurements = (): MeasurementsModel => ({
  columns: [
    { id: 'ch1', label: 'CH1', color: 'color-ch1' },
    { id: 'ch2', label: 'CH2', color: 'color-ch2' },
    { id: 'ch3', label: 'CH3', color: 'color-ch3' },
  ],
  rows: measurementRows(),
  accuracyNote: 'Volts accurate to about 2 to 3 ADC counts (placeholder wording).',
  pinnable: measurementRows().map(({ id, label }) => ({ id, label, pinned: true })),
  footnotes: [
    { key: 1, text: 'Reason text from the backend (example: not measurable at this setting).' },
    { key: 2, text: 'Reason text from the backend (example: fewer than one period).' },
    { key: 3, text: 'Reason text from the backend (example: not a two-level pulse).' },
  ],
  statistics: {
    title: 'CH1 Vpp · last 50 frames',
    headers: ['Now', 'Min', 'Mean', 'Max', 'σ'],
    values: ['1.91 V', '1.90 V', '1.91 V', '1.92 V', '4.1 mV'],
  },
});

export const mockGraph = (): MainWindowModel['graph'] => ({
  state: 'live',
  view: 'yt',
  running: true,
  dimmed: false,
  tags: [],
  selected: 1,
  channels: [
    { channel: 3, samples: triangleWave() },
    { channel: 2, samples: sineWave() },
    { channel: 1, samples: squareWave() },
  ],
  summary:
    'Yt display, 10 by 8 divisions. CH1 square wave 1.91 V peak to peak at 500 mV per division, CH2 sine 32 mV peak to peak at 20 mV per division, CH3 triangle at 2 V per division. Triggered on CH1 rising edge at 900 mV.',
});

type Analysis = Pick<MainWindowModel, 'views' | 'persistence' | 'peak' | 'roll' | 'recording' | 'autoset' | 'math'>;

const PERSISTENCE_OPTIONS: PersistenceModel['options'] = [
  { value: 'off', label: 'Off' },
  { value: '500ms', label: '0.5 s' },
  { value: '1s', label: '1 s' },
  { value: '2s', label: '2 s' },
  { value: 'infinite', label: 'Infinite' },
];

const channelOptions = (enabled: number[]) =>
  ([1, 2, 3, 4] as const).map((channel) => ({ channel, availability: { enabled: enabled.includes(channel) } }));

const mockViews = (): Analysis['views'] => ({
  tab: 'yt',
  tabAvailability: { yt: { enabled: true }, xy: { enabled: true }, fft: { enabled: true } },
  xy: {
    x: 1,
    y: 2,
    xOptions: channelOptions([1, 2, 3]),
    yOptions: channelOptions([1, 2, 3]),
    scales: { x: 'CH1 500 mV/div', y: 'CH2 200 mV/div' },
    readline: 'Fs 1.25 MS/s · straight segments',
  },
  fft: {
    window: 'hann',
    scale: 'dbv',
    available: { enabled: true },
    windows: [
      { value: 'rectangular', label: 'Rectangular' },
      { value: 'hann', label: 'Hann' },
      { value: 'hamming', label: 'Hamming' },
      { value: 'blackman', label: 'Blackman' },
    ],
    windowShort: 'Hann',
    windowLabel: 'Hann',
    ticksX: [],
    ticksY: [],
    readout: { span: '0 to 25 kHz', binSpacing: '12.21 Hz', topLabel: '+10 dBV', scale: '10 dB/div' },
    readline: 'Fs 50 kS/s · bin 12.21 Hz (Fs / N)',
  },
});

const mockMath = (): Analysis['math'] => ({
  row: {
    enabled: false,
    operation: 'subtract',
    sourceA: 1,
    sourceB: 2,
    scale: { value: '1', unit: 'V/div' },
    position: { value: '0.00', unit: 'div' },
    label: 'A−B',
    status: 'ok',
    availability: { enabled: true },
  },
  chip: '',
});

/** The analysis part of the mock: every feature available, nothing recording, no gaps. */
export const mockAnalysis = (): Analysis => ({
  views: mockViews(),
  persistence: {
    value: 'off',
    options: PERSISTENCE_OPTIONS,
    tag: '',
    available: { enabled: true },
    epoch: 0,
  },
  peak: {
    mode: 'auto',
    channel: 1,
    bin: 0,
    frequency: '1.001 kHz',
    level: '−6.9 dBV',
    available: { enabled: true },
    note: '',
    bins: 0,
  },
  roll: { active: false, window: '', gaps: [], gapCount: 0, announcement: '' },
  recording: {
    active: false,
    available: { enabled: false, reason: 'Recording needs roll mode (500 ms/div or slower)' },
  },
  autoset: { busy: false, noSignal: false, available: { enabled: true } },
  math: mockMath(),
});
