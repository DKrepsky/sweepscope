import { unavailableReason, type ModelInputs } from './select-common';
import type { MathModel } from './types';

/** Placeholder until the math row work package: disabled, empty, scale and position unmeasured. */
export function selectMath({ connection }: ModelInputs): MathModel {
  return {
    row: {
      enabled: false,
      operation: 'subtract',
      sourceA: 1,
      scale: { value: '-', unit: '' },
      position: { value: '-', unit: '' },
      label: '',
      status: 'ok',
      availability: { enabled: false, reason: unavailableReason(connection) },
    },
    chip: '',
  };
}
