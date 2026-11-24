import { formatQuantity } from '../formatting';
import { ROLL_WINDOW_SAMPLES } from '../waveform';
import type { ModelInputs } from './select-common';
import type { GapModel, RollModel } from './types';

const DIVISIONS = 10;

/** Seconds the window spans: ten divisions of the time base. */
const windowSeconds = ({ instrument }: ModelInputs): number =>
  instrument ? instrument.horizontal.timeBase.label.value * DIVISIONS : NaN;

function durationText(missed: number, seconds: number): string | undefined {
  const duration = (missed / ROLL_WINDOW_SAMPLES) * seconds;
  if (!Number.isFinite(duration) || duration <= 0) return undefined;
  const { text, unit } = formatQuantity(duration, 'seconds');
  return `${text} ${unit}`;
}

function gapsOf(inputs: ModelInputs): { model: GapModel; duration: string | undefined }[] {
  const seconds = windowSeconds(inputs);
  return (inputs.roll?.gaps ?? []).map(({ from, to, missed }) => {
    const duration = durationText(missed, seconds);
    return { model: { from, to, label: duration ? `GAP ${duration}` : 'GAP' }, duration };
  });
}

const INACTIVE: RollModel = { active: false, window: '', gaps: [], gapCount: 0, announcement: '' };

const announce = (duration: string | undefined) => `Data gap${duration ? ` of ${duration}` : ''}`;

/** The mode follows the backend; the window and gaps come from the roll window the stream feed owns. */
export function selectRoll(inputs: ModelInputs): RollModel {
  const { instrument, roll } = inputs;
  if (!instrument?.horizontal.roll) return INACTIVE;
  const gaps = gapsOf(inputs);
  const newest = gaps.at(-1);
  return {
    active: true,
    window: instrument.horizontal.windowText,
    gaps: gaps.map(({ model }) => model),
    gapCount: roll?.gapCount ?? 0,
    announcement: newest ? announce(newest.duration) : '',
  };
}
