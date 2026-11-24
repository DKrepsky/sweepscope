import type { ConnectionSnapshot, InstrumentState } from '../../api';
import {
  channelNumber,
  fractionFromCentre,
  NOT_MEASURED,
  toAvailability,
  toQuantity,
  unavailableReason,
} from './select-common';
import type { PulseModel, TriggerModel } from './types';

/** Empty until the accuracy wording is decided (PRD 5.2, question Q-E4); the group hides an empty note. */
const ACCURACY_NOTE = '';

type PulseState = NonNullable<InstrumentState['trigger']['pulse']>;

/** The error text is fixed wording around the width the backend still applies. */
function toPulse({ condition, width, verified }: PulseState, rejected: boolean): PulseModel {
  const applied = toQuantity(width);
  return {
    condition,
    width: applied,
    verified,
    ...(rejected && {
      error: `Invalid width. Enter a time such as 500 µs. Applied: ${applied.value} ${applied.unit}.`,
    }),
  };
}

export function selectTrigger(
  connection: ConnectionSnapshot | undefined,
  instrument: InstrumentState | undefined,
  pulseWidthRejected = false,
): TriggerModel {
  if (!instrument) {
    return {
      type: 'edge',
      source: 1,
      slope: 'rising',
      level: { value: NOT_MEASURED, unit: '' },
      levelPosition: 0.5,
      timePosition: 0.5,
      sweep: 'auto',
      levelTag: NOT_MEASURED,
      positionTag: NOT_MEASURED,
      accuracyNote: ACCURACY_NOTE,
      availability: { enabled: false, reason: unavailableReason(connection) },
    };
  }
  const { trigger, horizontal, controls } = instrument;
  return {
    type: trigger.kind,
    ...(trigger.pulse && { pulse: toPulse(trigger.pulse, pulseWidthRejected) }),
    source: channelNumber(trigger.source),
    slope: trigger.slope,
    level: toQuantity(trigger.level),
    levelPosition: fractionFromCentre(trigger.levelDivisions),
    timePosition: horizontal.triggerPositionFraction,
    sweep: trigger.sweep,
    levelTag: trigger.levelTag,
    positionTag: trigger.positionTag,
    accuracyNote: ACCURACY_NOTE,
    availability: toAvailability(controls.trigger),
  };
}
