import type { ConnectionSnapshot, InstrumentState, MeasurementsState } from '../../api';
import {
  channelNumber,
  fractionFromCentre,
  NOT_MEASURED,
  toAvailability,
  toQuantity,
  unavailableReason,
} from './select-common';
import type { XyAxes } from '../views';
import { aliasOf } from './select-warnings';
import type { ChannelModel, ChannelNumber, ScaleQuantity } from './types';

type ChannelState = InstrumentState['channels'][number];

const toScale = (range: ChannelState['range']): ScaleQuantity => ({
  ...toQuantity(range.label),
  step: { faster: toAvailability(range.faster), slower: toAvailability(range.slower) },
});

function toChannel(
  state: ChannelState,
  vertical: ChannelModel['availability'],
  selected: ChannelNumber,
  clipped: boolean,
): ChannelModel {
  const number = channelNumber(state.number);
  return {
    number,
    enabled: state.enabled,
    selected: number === selected,
    voltsPerDiv: toScale(state.range),
    position: toQuantity(state.position),
    coupling: state.coupling,
    bandwidthLimit: state.bandwidthLimit,
    probe: state.probeText.replace('x', '×'),
    inverted: state.invert,
    ...(clipped && { clipped }),
    groundPosition: fractionFromCentre(state.positionDivs),
    ...(!state.enable.enabled && state.enable.reason && { enableReason: state.enable.reason }),
    availability: vertical,
  };
}

function placeholderChannel(number: ChannelNumber, selected: ChannelNumber, reason: string): ChannelModel {
  const none = { enabled: false, reason };
  const unavailable = { value: NOT_MEASURED, unit: '' };
  return {
    number,
    enabled: false,
    selected: number === selected,
    voltsPerDiv: { ...unavailable, step: { faster: none, slower: none } },
    position: unavailable,
    coupling: 'dc',
    bandwidthLimit: false,
    probe: '1×',
    inverted: false,
    groundPosition: 0.5,
    enableReason: reason,
    availability: none,
  };
}

/** Alias badge and XY role, kept out of `toChannel` so each source of a badge stays in one place. */
function extras(
  number: number,
  measurements: MeasurementsState | undefined,
  xyRoles: XyAxes | undefined,
): Pick<ChannelModel, 'alias' | 'role'> {
  const alias = aliasOf(measurements, number);
  const role = xyRoles?.x === number ? 'x' : xyRoles?.y === number ? 'y' : undefined;
  return { ...(alias && { alias }), ...(role && { role }) };
}

/** `xyRoles` is given only while the XY view is shown. */
export function selectChannels(
  connection: ConnectionSnapshot | undefined,
  instrument: InstrumentState | undefined,
  selected: ChannelNumber,
  measurements?: MeasurementsState,
  xyRoles?: XyAxes,
): ChannelModel[] {
  if (instrument) {
    const vertical = toAvailability(instrument.controls.vertical);
    const isClipped = (number: number) =>
      measurements?.channels.some((m) => m.channel === number && m.clipped) ?? false;
    return instrument.channels.map((state) => ({
      ...toChannel(state, vertical, selected, isClipped(state.number)),
      ...extras(state.number, measurements, xyRoles),
    }));
  }
  const reason = unavailableReason(connection);
  return ([1, 2, 3, 4] as const).map((number) => placeholderChannel(number, selected, reason));
}
