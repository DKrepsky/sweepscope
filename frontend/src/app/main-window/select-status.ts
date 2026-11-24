import type { ConnectionSnapshot, InstrumentFacts, InstrumentState, MeasurementsState } from '../../api';
import { connectionChip } from '../instrument';
import { NOT_MEASURED } from './select-common';
import { aliasSummary } from './select-warnings';
import type { StatusModel } from './types';

type Fact = InstrumentFacts['sampleRate'];

const factText = (fact: Fact | undefined): string => (fact ? `${fact.text} ${fact.unit}`.trim() : NOT_MEASURED);

/** The record length in divisions is part of the backend's window text, "10.0 of 16.4 div". */
const recordSpan = (instrument: InstrumentState | undefined): string =>
  /\bof (.+)$/.exec(instrument?.horizontal.windowText ?? '')?.[1] ?? NOT_MEASURED;

/** Shown when no measurements slice exists yet; the wording itself is backend-owned. */
const NO_ACCURACY_NOTE = 'No measurement yet';

export function selectStatus(
  connection: ConnectionSnapshot | undefined,
  instrument: InstrumentState | undefined,
  facts: InstrumentFacts | undefined,
  renderer: string,
  measurements: MeasurementsState | undefined,
): StatusModel {
  const dead = facts?.deadTime;
  const alias = aliasSummary(measurements);
  return {
    connection: connectionChip(connection),
    sampleRate: {
      value: factText(facts?.sampleRate),
      tooltip: 'Effective sample rate for this time base and channel count',
      ...(alias && { alias }),
    },
    record: { points: factText(facts?.recordPoints), span: recordSpan(instrument) },
    capture: {
      rate: factText(facts?.captureRate),
      deadTime: dead ? `dead time ${factText(dead)}` : NOT_MEASURED,
      tooltip: 'Frames per second the USB cycle delivers, and the share of time the scope is actually capturing',
    },
    renderer,
    accuracy: measurements?.accuracyNote || NO_ACCURACY_NOTE,
  };
}
