import { NOT_MEASURED, toAvailability, unavailableReason, type ModelInputs } from './select-common';
import { fftAvailability, isLinear, levelStep, spectrumTicks, valueText, windowOf, WINDOWS } from './select-spectrum';
import type {
  Availability,
  AxisOption,
  ChannelNumber,
  FftModel,
  PersistenceModel,
  ScreenProblem,
  ViewsModel,
  XyModel,
} from './types';

const CHANNELS: ChannelNumber[] = [1, 2, 3, 4];

/** The tabs follow the backend's frame-view availability. */
function frameViews({ connection, instrument }: ModelInputs): Availability {
  return instrument
    ? toAvailability(instrument.controls.frameViews)
    : { enabled: false, reason: unavailableReason(connection) };
}

const isOn = ({ instrument }: ModelInputs, channel: ChannelNumber): boolean =>
  instrument?.channels.some((c) => c.number === channel && c.enabled) ?? false;

function axisOptions(inputs: ModelInputs, axis: 'x' | 'y'): AxisOption[] {
  const other = axis === 'x' ? inputs.xyAxes.y : inputs.xyAxes.x;
  return CHANNELS.map((channel) => {
    if (!isOn(inputs, channel)) return { channel, availability: { enabled: false, reason: 'Channel off' } };
    if (channel === other) {
      return { channel, availability: { enabled: false, reason: `Already used as ${axis === 'x' ? 'Y' : 'X'}` } };
    }
    return { channel, availability: { enabled: true } };
  });
}

function xyProblem(inputs: ModelInputs): ScreenProblem | undefined {
  if (!inputs.instrument) return undefined;
  const on = CHANNELS.filter((channel) => isOn(inputs, channel));
  if (on.length < 2) {
    const text = on[0] ? `Only CH${on[0]} is on. Enable another channel.` : 'No channel is on. Enable two channels.';
    return { title: 'XY needs two enabled channels', text };
  }
  const { x, y } = inputs.xyAxes;
  if (!isOn(inputs, x)) return { title: 'XY cannot draw', text: `CH${x} (X) is off.`, enable: x };
  if (!isOn(inputs, y)) return { title: 'XY cannot draw', text: `CH${y} (Y) is off.`, enable: y };
  return undefined;
}

const scaleOf = ({ instrument }: ModelInputs, channel: ChannelNumber): string => {
  const label = instrument?.channels.find((c) => c.number === channel)?.range.label;
  return `CH${channel} ${label ? valueText(label) : NOT_MEASURED}`;
};

const sampleRateText = ({ facts }: ModelInputs): string =>
  facts?.sampleRate ? valueText(facts.sampleRate) : NOT_MEASURED;

function selectXy(inputs: ModelInputs): XyModel {
  const { x, y } = inputs.xyAxes;
  const problem = xyProblem(inputs);
  return {
    x,
    y,
    xOptions: axisOptions(inputs, 'x'),
    yOptions: axisOptions(inputs, 'y'),
    scales: { x: scaleOf(inputs, x), y: scaleOf(inputs, y) },
    readline: `Fs ${sampleRateText(inputs)} · straight segments`,
    ...(problem && { problem }),
  };
}

const FFT_EMPTY: ScreenProblem = { title: 'No spectrum', text: 'Enable a channel to see its spectrum.' };

function fftReadout(inputs: ModelInputs): FftModel['readout'] {
  const { spectrum, spectrumFrame } = inputs;
  return {
    span: spectrum ? `0 to ${valueText(spectrum.span)}` : NOT_MEASURED,
    binSpacing: spectrum ? valueText(spectrum.binSpacing) : NOT_MEASURED,
    topLabel: spectrum?.topLabel ?? NOT_MEASURED,
    scale: spectrumFrame ? levelStep(isLinear(inputs), spectrumFrame.unitsPerDiv) : NOT_MEASURED,
  };
}

function selectFft(inputs: ModelInputs): FftModel {
  const { spectrum, instrument } = inputs;
  const available = fftAvailability(inputs);
  const window = windowOf(spectrum?.window);
  const noChannel = available.enabled && instrument !== undefined && !instrument.channels.some((c) => c.enabled);
  const readout = fftReadout(inputs);
  return {
    window: window.value,
    scale: spectrum?.scale ?? 'dbv',
    available,
    windows: WINDOWS.map(({ value, label }) => ({ value, label })),
    windowShort: window.short,
    windowLabel: window.label,
    ...spectrumTicks(inputs),
    readout,
    readline: `Fs ${sampleRateText(inputs)} · bin ${readout.binSpacing} (Fs / N)`,
    ...(noChannel && { problem: FFT_EMPTY }),
  };
}

export function selectViews(inputs: ModelInputs): ViewsModel {
  const views = frameViews(inputs);
  return {
    tab: inputs.view,
    tabAvailability: { yt: { enabled: true }, xy: views, fft: views },
    xy: selectXy(inputs),
    fft: selectFft(inputs),
  };
}

const PERSISTENCE_OPTIONS: PersistenceModel['options'] = [
  { value: 'off', label: 'Off' },
  { value: '500ms', label: '0.5 s' },
  { value: '1s', label: '1 s' },
  { value: '2s', label: '2 s' },
  { value: 'infinite', label: 'Infinite' },
];

const YT_ONLY = 'Persistence is available in Yt only';

export function selectPersistence(inputs: ModelInputs): PersistenceModel {
  const value = inputs.display?.persistence ?? 'off';
  const label = PERSISTENCE_OPTIONS.find((option) => option.value === value)?.label ?? 'Off';
  return {
    value,
    options: PERSISTENCE_OPTIONS,
    tag: value === 'off' ? '' : `PERSIST ${label}`,
    available: inputs.view === 'yt' ? frameViews(inputs) : { enabled: false, reason: YT_ONLY },
    epoch: inputs.persistenceEpoch,
  };
}
