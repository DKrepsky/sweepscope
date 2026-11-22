import type { ConnectionSnapshot, InstrumentState } from '../../api';
import {
  channelNumber,
  fractionFromCentre,
  NOT_MEASURED,
  toAvailability,
  toQuantity,
  unavailableReason,
} from './select-common';
import type { ChannelModel, ChannelNumber, ScaleQuantity } from './types';

type ChannelState = InstrumentState['channels'][number];

const toScale = (range: ChannelState['range']): ScaleQuantity => ({
  ...toQuantity(range.label),
  step: { faster: toAvailability(range.faster), slower: toAvailability(range.slower) },
});

function toChannel(state: ChannelState, vertical: ChannelModel['availability'], selected: ChannelNumber): ChannelModel {
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
    ...(state.unverified && { unverified: state.unverified }),
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

export function selectChannels(
  connection: ConnectionSnapshot | undefined,
  instrument: InstrumentState | undefined,
  selected: ChannelNumber,
): ChannelModel[] {
  if (instrument) {
    const vertical = toAvailability(instrument.controls.vertical);
    return instrument.channels.map((state) => toChannel(state, vertical, selected));
  }
  const reason = unavailableReason(connection);
  return ([1, 2, 3, 4] as const).map((number) => placeholderChannel(number, selected, reason));
}
