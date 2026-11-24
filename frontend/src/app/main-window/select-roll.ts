import type { ModelInputs } from './select-common';
import type { RollModel } from './types';

/** Placeholder until the roll work package: the mode follows the backend, no window text or gap is shown. */
export function selectRoll({ instrument }: ModelInputs): RollModel {
  return { active: instrument?.horizontal.roll ?? false, window: '', gaps: [], gapCount: 0 };
}
