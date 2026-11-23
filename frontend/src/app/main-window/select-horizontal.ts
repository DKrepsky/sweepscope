import type { ConnectionSnapshot, InstrumentFacts, InstrumentState } from '../../api';
import { NOT_MEASURED, toAvailability, toQuantity, unavailableReason } from './select-common';
import type { HorizontalModel, Interpolation } from './types';

const factText = (fact: InstrumentFacts['sampleRate'] | undefined): string =>
  fact ? `${fact.text} ${fact.unit}`.trim() : NOT_MEASURED;

const UNVERIFIED_TIME_BASE = 'Unverified (see accuracy note).';

export const rateNote = (facts: InstrumentFacts | undefined): string =>
  `Fs ${factText(facts?.sampleRate)} · Ts ${factText(facts?.sampleInterval)} · frame mode`;

const overviewOf = (horizontal: InstrumentState['horizontal']): HorizontalModel['overview'] => ({
  windowStart: horizontal.windowStartFraction,
  windowSize: horizontal.windowSizeFraction,
  triggerTick: horizontal.triggerTickFraction,
  summary: `Record overview: the visible window covers ${horizontal.windowText}`,
});

export function selectHorizontal(
  connection: ConnectionSnapshot | undefined,
  instrument: InstrumentState | undefined,
  facts: InstrumentFacts | undefined,
  interpolation: Interpolation,
  automaticInterpolation = false,
): HorizontalModel {
  if (!instrument) {
    const none = { enabled: false, reason: unavailableReason(connection) };
    return {
      timePerDiv: { value: NOT_MEASURED, unit: '', step: { faster: none, slower: none } },
      triggerPosition: { value: NOT_MEASURED, unit: '' },
      window: NOT_MEASURED,
      interpolation,
      automaticInterpolation,
      availability: none,
      rateNote: rateNote(facts),
      overview: { windowStart: 0.2, windowSize: 0.6, triggerTick: 0.5, summary: 'Record overview: no data' },
    };
  }
  const { horizontal, controls } = instrument;
  const { timeBase } = horizontal;
  return {
    timePerDiv: {
      ...toQuantity(timeBase.label),
      step: { faster: toAvailability(timeBase.faster), slower: toAvailability(timeBase.slower) },
    },
    ...(!timeBase.verified && { unverified: UNVERIFIED_TIME_BASE }),
    triggerPosition: toQuantity(horizontal.triggerPosition),
    window: horizontal.windowText,
    interpolation,
    automaticInterpolation,
    availability: toAvailability(controls.horizontal),
    rateNote: rateNote(facts),
    overview: overviewOf(horizontal),
  };
}
