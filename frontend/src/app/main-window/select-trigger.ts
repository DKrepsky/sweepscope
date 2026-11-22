import type { ConnectionSnapshot, InstrumentState } from '../../api';
import {
  channelNumber,
  fractionFromCentre,
  NOT_MEASURED,
  toAvailability,
  toQuantity,
  unavailableReason,
} from './select-common';
import type { TriggerModel } from './types';

/** Placeholder until the accuracy wording is decided (PRD 5.2, question Q-E4). */
const ACCURACY_NOTE = 'Accuracy note slot (wording TBD)';

export function selectTrigger(
  connection: ConnectionSnapshot | undefined,
  instrument: InstrumentState | undefined,
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
    type: 'edge',
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
