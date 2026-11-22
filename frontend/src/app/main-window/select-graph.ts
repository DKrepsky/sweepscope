import type { ConnectionSnapshot, InstrumentState } from '../../api';
import { graphState } from '../instrument';
import type { DecodedFrame } from '../waveform';
import type { ChannelNumber, GraphModel } from './types';

const SCREEN = 'Yt display, 10 by 8 divisions.';

/** The selected trace is drawn last so it lies on top. */
function drawOrder(frame: DecodedFrame, selected: ChannelNumber): GraphModel['channels'] {
  const traces = frame.channels.map(({ channel, samples }) => ({ channel, samples }));
  return [...traces.filter((t) => t.channel !== selected), ...traces.filter((t) => t.channel === selected)];
}

function describe(frame: DecodedFrame | undefined, instrument: InstrumentState | undefined): string {
  if (!frame || !instrument) return `${SCREEN} No signal captured yet.`;
  const traces = frame.channels.map(({ channel }) => {
    const range = instrument.channels.find((c) => c.number === channel)?.range.label;
    return range ? `CH${channel} at ${range.text} ${range.unit}` : `CH${channel}`;
  });
  const { trigger } = instrument;
  return `${SCREEN} ${traces.join(', ')}. Trigger on CH${trigger.source}, ${trigger.slope} edge at ${trigger.level.text} ${trigger.level.unit}.`;
}

export function selectGraph(
  connection: ConnectionSnapshot | undefined,
  instrument: InstrumentState | undefined,
  frame: DecodedFrame | undefined,
  selected: ChannelNumber,
): GraphModel {
  return {
    state: graphState(connection, instrument, frame !== undefined),
    channels: frame ? drawOrder(frame, selected) : [],
    selected,
    summary: describe(frame, instrument),
  };
}
