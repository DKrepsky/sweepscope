import type { SpectrumFrameInput } from '../views';
import { NOT_MEASURED, SCREEN_VERTICAL_DIVISIONS, type ModelInputs } from './select-common';
import { fftAvailability, frequencyText, isLinear, levelText, valueText } from './select-spectrum';
import type { Availability, PeakMarkerModel, PeakModel } from './types';

type Trace = SpectrumFrameInput['channels'][number];

interface Located {
  frame: SpectrumFrameInput;
  trace: Trace;
  bin: number;
}

const clamp01 = (value: number) => Math.min(1, Math.max(0, value));

/** Auto follows the backend's peak bin; Manual keeps the bin the user placed; Off shows none. */
function markerBin({ peak }: ModelInputs, trace: Trace, bins: number): number | undefined {
  if (peak.mode === 'off') return undefined;
  const bin = peak.mode === 'auto' ? trace.peakBin : peak.bin;
  return bin === undefined ? undefined : Math.min(bins - 1, Math.max(0, bin));
}

function locate(inputs: ModelInputs): Located | undefined {
  const frame = inputs.spectrumFrame;
  const trace = frame?.channels.find((c) => c.channel === inputs.peak.channel);
  if (!frame || !trace) return undefined;
  const bin = markerBin(inputs, trace, frame.bins);
  return bin === undefined ? undefined : { frame, trace, bin };
}

const levelAt = (inputs: ModelInputs, { trace, bin }: Located) => levelText(isLinear(inputs), trace.levels[bin] ?? NaN);

function placed(inputs: ModelInputs, found: Located): PeakMarkerModel {
  const { frame, trace, bin } = found;
  const last = Math.max(1, frame.bins - 1);
  return {
    position: bin / last,
    level: clamp01(1 - (trace.divisions[bin] ?? 0) / SCREEN_VERTICAL_DIVISIONS),
    step: 1 / last,
    valueText: `${frequencyText(bin, frame.binHz)} ${levelAt(inputs, found)}`,
  };
}

function availability(inputs: ModelInputs): Availability {
  const fft = fftAvailability(inputs);
  if (!fft.enabled) return fft;
  return inputs.spectrumFrame ? { enabled: true } : { enabled: false, reason: 'No spectrum yet' };
}

const noteOf = ({ spectrum }: ModelInputs): string =>
  spectrum ? `Frequency resolution is one bin (${valueText(spectrum.binSpacing)}).` : '';

/** The marker, the panel and the tag read one `Located` bin, so their numbers cannot differ. */
export function selectPeak(inputs: ModelInputs): PeakModel {
  const found = locate(inputs);
  return {
    ...inputs.peak,
    bin: found?.bin ?? inputs.peak.bin,
    frequency: found ? frequencyText(found.bin, found.frame.binHz) : NOT_MEASURED,
    level: found ? levelAt(inputs, found) : NOT_MEASURED,
    available: availability(inputs),
    bins: inputs.spectrumFrame?.bins ?? 0,
    note: noteOf(inputs),
    ...(found && { marker: placed(inputs, found) }),
  };
}
