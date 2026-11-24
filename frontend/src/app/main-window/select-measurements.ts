import type { MeasurementsState } from '../../api';
import type { GraphModel, MeasurementCellModel, MeasurementsModel } from './types';

type ChannelMeasurements = MeasurementsState['channels'][number];
type Measurement = ChannelMeasurements['measurements'][number];

type Row = MeasurementsModel['rows'][number];

const EMPTY: MeasurementsModel = { columns: [], rows: [], footnotes: [], accuracyNote: '', pinnable: [] };

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

function toCell(
  measurement: Measurement,
  unverified: string | null,
  keyFor: (text: string) => number,
): MeasurementCellModel {
  if (measurement.invalid) return { state: 'invalid', text: '-', footnote: keyFor(measurement.invalid.text) };
  const { text, unit } = measurement.value ?? { text: '-', unit: '' };
  const shown = { text, ...(unit && { unit }) };
  return unverified ? { ...shown, state: 'unverified', tooltip: unverified } : { ...shown, state: 'valid' };
}

function toRows(channels: ChannelMeasurements[], pinned: ReadonlySet<string>, keyFor: (text: string) => number): Row[] {
  const shown = channels[0]?.measurements.filter((measurement) => pinned.has(measurement.id)) ?? [];
  return shown.map(({ id, label }) => ({
    id,
    label,
    cells: Object.fromEntries(
      channels.flatMap((channel) => {
        const measurement = channel.measurements.find((other) => other.id === id);
        return measurement ? [[`ch${channel.channel}`, toCell(measurement, channel.unverified, keyFor)]] : [];
      }),
    ),
  }));
}

export function selectMeasurements(
  measurements: MeasurementsState | undefined,
  pinned: readonly string[],
  graph: GraphModel['state'],
): MeasurementsModel {
  if (!measurements || measurements.channels.length === 0 || graph === 'empty') return EMPTY;
  const chosen = new Set(pinned);
  const { notes, keyFor } = footnoteRegistry();
  const rows = toRows(measurements.channels, chosen, keyFor);
  return {
    columns: measurements.channels.map(({ channel }) => ({
      id: `ch${channel}`,
      label: `CH${channel}`,
      color: `color-ch${channel}`,
    })),
    rows,
    footnotes: notes,
    ...(graph === 'frozen' && { frozenLabel: 'Frozen' as const }),
    accuracyNote: measurements.accuracyNote,
    pinnable: (measurements.channels[0]?.measurements ?? []).map(({ id, label }) => ({
      id,
      label,
      pinned: chosen.has(id),
    })),
  };
}
