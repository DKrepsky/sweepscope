import type { MeasurementsState } from '../../api';
import { aliasOf } from './select-warnings';
import type { GraphModel, MeasurementCellModel, MeasurementsModel } from './types';

type ChannelMeasurements = MeasurementsState['channels'][number];
type Measurement = ChannelMeasurements['measurements'][number];

type Row = MeasurementsModel['rows'][number];

const EMPTY: MeasurementsModel = { columns: [], rows: [], footnotes: [], pinnable: [] };
const MATH_CHANNEL = 5;
const ALIAS_AFFECTED = new Set<string>(['frequency', 'period']);
const ROLL_NOTE = 'Measurements are not available in roll mode. Use a faster time base to measure.';

/** Numbers each distinct reason text once, in order of first use. */
function footnoteRegistry() {
  const notes: MeasurementsModel['footnotes'] = [];
  const keyFor = (text: string): number => {
    const known = notes.find((note) => note.text === text);
    if (known) return known.key;
    notes.push({ key: notes.length + 1, text });
    return notes.length;
  };
  return { notes, keyFor };
}

/** The backend measures the math trace in volts; a product of two channels is in volts squared, prefix kept. */
function withMathUnit(unit: string, mathUnit: string | undefined): string {
  if (!mathUnit || mathUnit === 'V' || !unit.endsWith('V')) return unit;
  return `${unit.slice(0, -1)}${mathUnit}`;
}

interface CellContext {
  keyFor: (text: string) => number;
  aliasMessage?: string;
  mathUnit?: string;
}

function toCell(measurement: Measurement, context: CellContext): MeasurementCellModel {
  const { keyFor, aliasMessage, mathUnit } = context;
  if (measurement.invalid) return { state: 'invalid', text: '-', footnote: keyFor(measurement.invalid.text) };
  const { text, unit } = measurement.value ?? { text: '-', unit: '' };
  const shownUnit = withMathUnit(unit, mathUnit);
  const shown = { text, ...(shownUnit && { unit: shownUnit }) };
  const suspect = aliasMessage !== undefined && ALIAS_AFFECTED.has(measurement.id);
  const flagged = suspect ? { ...shown, warning: true, footnote: keyFor(aliasMessage) } : shown;
  return { ...flagged, state: 'valid' };
}

const columnId = (channel: number) => `ch${channel}`;

interface RowContext {
  pinned: ReadonlySet<string>;
  keyFor: (text: string) => number;
  measurements: MeasurementsState;
  mathUnit?: string;
}

function cellContext(channel: ChannelMeasurements, { keyFor, measurements, mathUnit }: RowContext): CellContext {
  const aliasMessage = aliasOf(measurements, channel.channel)?.message;
  return {
    keyFor,
    ...(aliasMessage && { aliasMessage }),
    ...(channel.channel === MATH_CHANNEL && mathUnit && { mathUnit }),
  };
}

function toRows(context: RowContext): Row[] {
  const channels = context.measurements.channels;
  const shown = channels[0]?.measurements.filter((measurement) => context.pinned.has(measurement.id)) ?? [];
  return shown.map(({ id, label }) => ({
    id,
    label,
    cells: Object.fromEntries(
      channels.flatMap((channel) => {
        const measurement = channel.measurements.find((other) => other.id === id);
        return measurement ? [[columnId(channel.channel), toCell(measurement, cellContext(channel, context))]] : [];
      }),
    ),
  }));
}

function toColumn(measurements: MeasurementsState, channel: number): MeasurementsModel['columns'][number] {
  const alias = aliasOf(measurements, channel);
  const math = channel === MATH_CHANNEL;
  return {
    id: columnId(channel),
    label: math ? 'M' : `CH${channel}`,
    color: math ? 'color-math' : `color-ch${channel}`,
    ...(alias && { alias }),
  };
}

const hasValues = (
  measurements: MeasurementsState | undefined,
  graph: GraphModel['state'],
): measurements is MeasurementsState => !!measurements && measurements.channels.length > 0 && graph !== 'empty';

export interface MeasurementContext {
  /** Unit the math state reports for the math trace (`V` or `V²`). */
  mathUnit?: string;
  roll?: boolean;
}

export function selectMeasurements(
  measurements: MeasurementsState | undefined,
  pinned: readonly string[],
  graph: GraphModel['state'],
  { mathUnit, roll = false }: MeasurementContext = {},
): MeasurementsModel {
  if (roll) return { ...EMPTY, unavailableNote: ROLL_NOTE };
  if (!hasValues(measurements, graph)) return EMPTY;
  const chosen = new Set(pinned);
  const { notes, keyFor } = footnoteRegistry();
  const rows = toRows({ pinned: chosen, keyFor, measurements, ...(mathUnit && { mathUnit }) });
  return {
    columns: measurements.channels.map(({ channel }) => toColumn(measurements, channel)),
    rows,
    footnotes: notes,
    ...(graph === 'frozen' && { frozenLabel: 'Frozen' as const }),
    pinnable: (measurements.channels[0]?.measurements ?? []).map(({ id, label }) => ({
      id,
      label,
      pinned: chosen.has(id),
    })),
  };
}
