import { toAvailability, unavailableReason, type ModelInputs } from './select-common';
import type { Availability, AxisOption, ChannelNumber, PersistenceModel, ViewsModel } from './types';

const CHANNELS: ChannelNumber[] = [1, 2, 3, 4];

/** The tabs follow the backend's frame-view availability; XY and FFT detail arrives with their screens. */
function frameViews({ connection, instrument }: ModelInputs): Availability {
  return instrument
    ? toAvailability(instrument.controls.frameViews)
    : { enabled: false, reason: unavailableReason(connection) };
}

function axisOptions({ instrument }: ModelInputs): AxisOption[] {
  return CHANNELS.map((channel) => ({
    channel,
    availability: { enabled: instrument?.channels.some((c) => c.number === channel && c.enabled) ?? false },
  }));
}

export function selectViews(inputs: ModelInputs): ViewsModel {
  const views = frameViews(inputs);
  const options = axisOptions(inputs);
  return {
    tab: inputs.view,
    tabAvailability: { yt: { enabled: true }, xy: views, fft: views },
    xy: { x: inputs.xyAxes.x, y: inputs.xyAxes.y, xOptions: options, yOptions: options },
    fft: {
      window: 'hann',
      scale: 'dbv',
      available: views,
      ticksX: [],
      ticksY: [],
      readout: { span: '', binSpacing: '', topLabel: '' },
    },
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
