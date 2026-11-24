import { formatDbv, formatFrequency, formatQuantity } from '../formatting';
import { toAvailability, unavailableReason, type FormattedValue, type ModelInputs } from './select-common';
import type { Availability, SpectrumWindowName } from './types';

export const WINDOWS: { value: SpectrumWindowName; label: string; short: string }[] = [
  { value: 'rectangular', label: 'Rectangular', short: 'Rect' },
  { value: 'hann', label: 'Hann', short: 'Hann' },
  { value: 'hamming', label: 'Hamming', short: 'Hamming' },
  { value: 'blackman', label: 'Blackman', short: 'Blackman' },
];

export const windowOf = (name: SpectrumWindowName | undefined) => WINDOWS.find((w) => w.value === name) ?? WINDOWS[1]!;

export const valueText = ({ text, unit }: FormattedValue): string => `${text} ${unit}`.trim();

/** Why the spectrum cannot be shown: the backend reason for roll or no scope, else the spectrum's own availability. */
export function fftAvailability({ connection, instrument, spectrum }: ModelInputs): Availability {
  if (!instrument) return { enabled: false, reason: unavailableReason(connection) };
  if (!instrument.controls.frameViews.enabled) return toAvailability(instrument.controls.frameViews);
  return spectrum ? toAvailability(spectrum.availability) : { enabled: true };
}

/** The backend counts Y ticks from the top edge, the graph from the bottom. */
export function spectrumTicks({ spectrum }: Pick<ModelInputs, 'spectrum'>) {
  return {
    ticksX: (spectrum?.ticksX ?? []).map(({ at, text }) => ({ at, text })),
    ticksY: (spectrum?.ticksY ?? []).map(({ at, text }) => ({ at: 1 - at, text })),
  };
}

export const isLinear = ({ spectrum }: Pick<ModelInputs, 'spectrum'>) => spectrum?.scale === 'linear';

/** dBV or volts rms, whichever the scale shows. */
export function levelText(linear: boolean, level: number): string {
  if (!linear) return formatDbv(level);
  const { text, unit } = formatQuantity(level, 'volts');
  return `${text} ${unit}`;
}

export const frequencyText = (bin: number, binHz: number): string => formatFrequency(bin * binHz);

/** The size of one division on the level axis, such as `10 dB/div` or `200 mV/div`. */
export function levelStep(linear: boolean, unitsPerDiv: number): string {
  if (!linear) return `${unitsPerDiv} dB/div`;
  const { text, unit } = formatQuantity(unitsPerDiv, 'volts');
  return `${text} ${unit}/div`;
}

export function spectrumSummary(i: ModelInputs): string {
  const { spectrum } = i;
  if (!spectrum) return 'FFT display';
  return `FFT display, ${windowOf(spectrum.window).label} window, bin spacing ${valueText(spectrum.binSpacing)}.`;
}
