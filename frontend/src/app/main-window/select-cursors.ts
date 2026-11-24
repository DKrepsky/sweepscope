import type { InstrumentState } from '../../api';
import { formatQuantity, type Quantity } from '../formatting';
import type { CursorId, CursorsState } from '../cursors';
import { NOT_MEASURED, SCREEN_VERTICAL_DIVISIONS } from './select-common';
import type { ChannelNumber, CursorLineModel, CursorsModel, CursorValue, GraphModel } from './types';

const SCREEN_HORIZONTAL_DIVISIONS = 10;
const ORIGIN_NOTE = 'X origin: left edge of the screen, not the trigger.';
const NO_WAVEFORM = { enabled: false, reason: 'No waveform' };

const text = (value: number | undefined, quantity: Quantity): string => {
  if (value === undefined) return NOT_MEASURED;
  const { text: number, unit } = formatQuantity(value, quantity);
  return unit && number !== NOT_MEASURED ? `${number} ${unit}` : number;
};

const difference = (first: number | undefined, second: number | undefined): number | undefined =>
  first === undefined || second === undefined ? undefined : second - first;

interface Scales {
  secondsPerDiv: number | undefined;
  /** Undefined while the source channel is off or unknown: its Y readouts are not measurable. */
  volts: { perDiv: number; positionDivs: number } | undefined;
}

function scalesOf(instrument: InstrumentState | undefined, source: ChannelNumber): Scales {
  const channel = instrument?.channels.find((candidate) => candidate.number === source);
  return {
    secondsPerDiv: instrument?.horizontal.timeBase.label.value,
    volts: channel?.enabled ? { perDiv: channel.range.label.value, positionDivs: channel.positionDivs } : undefined,
  };
}

const seconds = (fraction: number | undefined, { secondsPerDiv }: Scales): number | undefined =>
  fraction === undefined || secondsPerDiv === undefined
    ? undefined
    : fraction * SCREEN_HORIZONTAL_DIVISIONS * secondsPerDiv;

const volts = (fractionFromTop: number | undefined, { volts }: Scales): number | undefined => {
  if (fractionFromTop === undefined || volts === undefined) return undefined;
  const divisionsAboveCentre = (0.5 - fractionFromTop) * SCREEN_VERTICAL_DIVISIONS;
  return (divisionsAboveCentre - volts.positionDivs) * volts.perDiv;
};

const reciprocal = (interval: number | undefined): number | undefined =>
  interval === undefined || interval === 0 ? undefined : 1 / Math.abs(interval);

function readouts(cursors: CursorsState, scales: Scales): Record<CursorId, number | undefined> {
  return {
    x1: seconds(cursors.x1, scales),
    x2: seconds(cursors.x2, scales),
    y1: volts(cursors.y1, scales),
    y2: volts(cursors.y2, scales),
  };
}

function valuesOf(measured: Record<CursorId, number | undefined>): CursorValue[] {
  const deltaX = difference(measured.x1, measured.x2);
  const deltaY = difference(measured.y1, measured.y2);
  return [
    { label: 'X1', value: text(measured.x1, 'seconds') },
    { label: 'Y1', value: text(measured.y1, 'volts') },
    { label: 'X2', value: text(measured.x2, 'seconds') },
    { label: 'Y2', value: text(measured.y2, 'volts') },
    { label: 'ΔX', value: text(deltaX, 'seconds'), emphasised: true },
    { label: 'ΔY', value: text(deltaY, 'volts'), emphasised: true },
    { label: '1/ΔX', value: text(reciprocal(deltaX), 'hertz') },
  ];
}

const AXIS: Record<CursorId, 'x' | 'y'> = { x1: 'x', x2: 'x', y1: 'y', y2: 'y' };
const ORDER: CursorId[] = ['x1', 'x2', 'y1', 'y2'];

function linesOf(cursors: CursorsState, measured: Record<CursorId, number | undefined>): CursorLineModel[] {
  return ORDER.flatMap((id) => {
    const position = cursors[id];
    if (position === undefined) return [];
    const tag = `${id.toUpperCase()} ${text(measured[id], AXIS[id] === 'x' ? 'seconds' : 'volts')}`;
    return [{ id, axis: AXIS[id], position, tag, valueText: tag }];
  });
}

function deltaOf(cursors: CursorsState, measured: Record<CursorId, number | undefined>): CursorsModel['delta'] {
  if (cursors.x1 === undefined || cursors.x2 === undefined) return undefined;
  return {
    tag: `ΔX ${text(difference(measured.x1, measured.x2), 'seconds')}`,
    position: (cursors.x1 + cursors.x2) / 2,
  };
}

export const selectCursors: (
  cursors: CursorsState,
  instrument: InstrumentState | undefined,
  graph: GraphModel['state'],
  selected: ChannelNumber,
) => CursorsModel = (cursors, instrument, graph, selected) => {
  const measured = readouts(cursors, scalesOf(instrument, selected));
  const delta = deltaOf(cursors, measured);
  return {
    enabled: Object.keys(cursors).length > 0,
    available: graph === 'empty' ? NO_WAVEFORM : { enabled: true },
    source: selected,
    values: valuesOf(measured),
    originNote: ORIGIN_NOTE,
    lines: linesOf(cursors, measured),
    ...(delta && { delta }),
  };
};
