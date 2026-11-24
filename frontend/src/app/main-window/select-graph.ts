import type { ConnectionSnapshot, InstrumentState } from '../../api';
import { graphState } from '../instrument';
import type { DecodedFrame, TraceChannel } from '../waveform';
import type { ChannelNumber, GraphModel } from './types';

const SCREEN = 'Yt display, 10 by 8 divisions.';

/** The math trace (5) waits for the graph component to draw it; until then only the four inputs reach the screen. */
const isInput = <T extends { channel: TraceChannel }>(trace: T): trace is T & { channel: ChannelNumber } =>
  trace.channel !== 5;

/** The selected trace is drawn last so it lies on top. */
function drawOrder(frame: DecodedFrame, selected: ChannelNumber): GraphModel['channels'] {
  const traces = frame.channels.filter(isInput).map(({ channel, samples }) => ({ channel, samples }));
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
