import type { ConnectionSnapshot, InstrumentFacts, InstrumentState, MeasurementsState } from '../../api';
import { NOT_MEASURED } from './select-common';
import { aliasSummary } from './select-warnings';
import type { StatusModel } from './types';

type Fact = InstrumentFacts['sampleRate'];

const factText = (fact: Fact | undefined): string => (fact ? `${fact.text} ${fact.unit}`.trim() : NOT_MEASURED);

/** The record length in divisions is part of the backend's window text, "10.0 of 16.4 div". */
const recordSpan = (instrument: InstrumentState | undefined): string =>
  /\bof (.+)$/.exec(instrument?.horizontal.windowText ?? '')?.[1] ?? NOT_MEASURED;

const DETAIL_SEPARATOR = ' · ';

const joined = (parts: (string | null | undefined)[]): string => parts.filter(Boolean).join(DETAIL_SEPARATOR);

function readyConnection(connection: ConnectionSnapshot): StatusModel['connection'] {
  const identity = connection.identity;
  const firmware = identity?.firmware;
  const detail = connection.simulatorActive
    ? joined(['Simulator', identity?.model])
    : joined([identity?.model, firmware && `FW ${firmware}`, identity?.fpgaVersion && `FPGA ${identity.fpgaVersion}`]);
  return { state: 'Ready', detail, tooltip: firmware ? `Firmware ${firmware}` : '' };
}

export function selectConnection(connection: ConnectionSnapshot | undefined): StatusModel['connection'] {
  switch (connection?.state) {
    case 'ready':
      return readyConnection(connection);
    case 'disconnected':
      return { state: 'Disconnected', detail: 'No scope found. Connect the scope', tooltip: '' };
    case 'error':
      return { state: 'Error', detail: connection.issue?.message ?? 'Connection failed', tooltip: '' };
    default:
      return { state: 'Connecting', detail: 'Waiting for the scope to settle', tooltip: '' };
  }
}

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
    connection: selectConnection(connection),
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
  };
}
