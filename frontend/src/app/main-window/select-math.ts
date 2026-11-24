import type { InstrumentState } from '../../api';
import { channelNumber, toAvailability, toQuantity, unavailableReason, type ModelInputs } from './select-common';
import type { MathModel, MathRowModel } from './types';

type MathState = InstrumentState['math'];

function rowStatus({ status }: MathState): Pick<MathRowModel, 'status' | 'statusDetail'> {
  if (status.kind === 'mixed') return { status: 'mixed', statusDetail: status.detail };
  if (status.kind === 'unavailable') return { status: 'unavailable', statusDetail: status.reason };
  return { status: 'ok' };
}

function toRow(math: MathState, availability: MathRowModel['availability']): MathRowModel {
  return {
    enabled: math.enabled,
    operation: math.operation,
    sourceA: channelNumber(math.sourceA),
    ...(math.sourceB !== null && { sourceB: channelNumber(math.sourceB) }),
    scale: {
      ...toQuantity(math.scale.label),
      step: { faster: toAvailability(math.scale.faster), slower: toAvailability(math.scale.slower) },
    },
    position: toQuantity(math.position),
    label: math.label,
    ...rowStatus(math),
    availability,
  };
}

/** The chip states the scale or, when the trace cannot be drawn, the reason, so it is visible without hovering. */
function chipText({ enabled, status, label, scale }: MathState): string {
  if (!enabled) return '';
  if (status.kind === 'unavailable') return `M  ${label}  unavailable: ${status.reason}`;
  return `M  ${label}  ${scale.label.text} ${scale.label.unit}`;
}

function withoutInstrument(reason: string): MathModel {
  const empty = { value: '-', unit: '' };
  const blocked = { enabled: false, reason };
  return {
    row: {
      enabled: false,
      operation: 'subtract',
      sourceA: 1,
      scale: { ...empty, step: { faster: blocked, slower: blocked } },
      position: empty,
      label: '',
      status: 'ok',
      availability: blocked,
    },
    chip: '',
  };
}

export function selectMath({ connection, instrument }: ModelInputs): MathModel {
  if (!instrument) return withoutInstrument(unavailableReason(connection));
  const { math, controls } = instrument;
  const reason = math.status.kind === 'unavailable' ? math.status.reason : undefined;
  return {
    row: toRow(math, toAvailability(controls.vertical)),
    chip: chipText(math),
    ...(reason && { unavailableReason: reason }),
  };
}
