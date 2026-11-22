import type { GraphFrame } from '../../ui/graph/graph-frame';
import type { MeasurementColumn, MeasurementFootnote, MeasurementRow } from '../../ui/measurement-matrix';

const SAMPLES = 400;
const CENTRE = 4;

export const ILLUSTRATIVE = 'All numbers and texts on this page are illustrative placeholders, not hardware facts.';

function wave(shape: (phase: number) => number, cycles: number, amplitude: number, offset: number): number[] {
  return Array.from({ length: SAMPLES }, (_, i) => CENTRE + offset - amplitude * shape((i / SAMPLES) * cycles));
}

const sine = (phase: number) => Math.sin(phase * 2 * Math.PI);
const square = (phase: number) => (phase % 1 < 0.5 ? 1 : -1);

export const liveFrame: GraphFrame = {
  state: 'live',
  selected: 1,
  channels: [
    { channel: 1, samples: wave(sine, 4, 1.5, 0) },
    { channel: 2, samples: wave(square, 4, 1, 1.2) },
    { channel: 3, samples: wave(sine, 2, 0.8, -1.4) },
    { channel: 4, samples: wave(sine, 7, 4.6, 0) },
  ],
};

export const frozenFrame: GraphFrame = { ...liveFrame, state: 'frozen', channels: liveFrame.channels.slice(0, 1) };

export const gapFrame: GraphFrame = {
  state: 'live',
  channels: [{ channel: 1, samples: wave(sine, 3, 1.2, 0) }],
};

export const emptyFrame: GraphFrame = { state: 'empty', channels: [] };

export const matrixColumns: MeasurementColumn[] = [
  { id: 'ch1', label: 'CH1', color: 'color-ch1' },
  { id: 'ch2', label: 'CH2', color: 'color-ch2' },
  { id: 'ch3', label: 'CH3', color: 'color-ch3' },
  { id: 'ch4', label: 'CH4', color: 'color-text-muted' },
];

export const matrixRows: MeasurementRow[] = [
  { id: 'valid', label: 'Valid', cells: { ch1: { state: 'valid', text: '1.000', unit: 'kHz' } } },
  {
    id: 'unverified',
    label: 'Unverified',
    cells: { ch2: { state: 'unverified', text: '32.0', unit: 'mV', tooltip: 'Range not verified (illustrative)' } },
  },
  {
    id: 'invalid',
    label: 'Invalid',
    cells: { ch3: { state: 'invalid', text: '', footnote: 2, tooltip: 'Not enough periods on screen (illustrative)' } },
  },
  { id: 'stale', label: 'Stale', cells: { ch1: { state: 'stale', text: '1.000', unit: 'kHz' } } },
];

export const matrixFootnotes: MeasurementFootnote[] = [
  { key: 2, text: 'Not enough periods on screen (illustrative).' },
];

export const menuItems = [
  { id: 'level', label: 'Set trigger level here' },
  { id: 'x', label: 'Add X cursor here', badge: 'Exp' },
  { id: 'y', label: 'Add Y cursor here', disabled: true, disabledReason: 'No waveform' },
];
