import { graphState, isAcquiring } from '../instrument';
import { channelNumber, type ModelInputs } from './select-common';
import { spectrumTicks, valueText } from './select-spectrum';
import type {
  ChannelNumber,
  GraphModel,
  GraphTag,
  PersistenceModel,
  PersistenceValue,
  RollModel,
  ViewsModel,
} from './types';

const MATH_CHANNEL = 5;
const SCREEN = 'display, 10 by 8 divisions.';

const DECAY_MS: Record<Exclude<PersistenceValue, 'off'>, number | 'infinite'> = {
  '500ms': 500,
  '1s': 1000,
  '2s': 2000,
  infinite: 'infinite',
};

interface Source {
  channel: number;
  samples: ArrayLike<number>;
}

type Shown = Pick<GraphModel, 'channels' | 'math'>;

/** The selected trace is drawn last so it lies on top; channel 5 is the math trace. */
function drawOrder(sources: readonly Source[], selected: ChannelNumber): Shown {
  const math = sources.find((source) => source.channel === MATH_CHANNEL)?.samples;
  const traces = sources
    .filter((source) => source.channel !== MATH_CHANNEL)
    .map(({ channel, samples }) => ({ channel: channelNumber(channel), samples }));
  return {
    channels: [...traces.filter((t) => t.channel !== selected), ...traces.filter((t) => t.channel === selected)],
    ...(math && { math: { samples: math } }),
  };
}

const sourcesOf = ({ view, frame, roll, spectrumFrame }: ModelInputs, rolling: boolean): readonly Source[] => {
  if (view === 'xy') return [];
  if (view === 'fft') return (spectrumFrame?.channels ?? []).map((c) => ({ ...c, samples: c.divisions }));
  return rolling ? (roll?.channels ?? []) : (frame?.channels ?? []);
};

function xyTrace(inputs: ModelInputs, views: ViewsModel): GraphModel['xy'] {
  if (inputs.view !== 'xy' || views.xy.problem || !inputs.frame) return undefined;
  const { x, y } = views.xy;
  const samplesOf = (channel: ChannelNumber) => inputs.frame?.channels.find((c) => c.channel === channel)?.samples;
  const xs = samplesOf(x);
  const ys = samplesOf(y);
  return xs && ys ? { x: xs, y: ys, xChannel: x, yChannel: y } : undefined;
}

function persistenceOf(inputs: ModelInputs, rolling: boolean): GraphModel['persistence'] {
  const value = inputs.display?.persistence ?? 'off';
  if (value === 'off' || inputs.view !== 'yt' || rolling || !inputs.instrument) return undefined;
  return { decayMs: DECAY_MS[value], epoch: inputs.persistenceEpoch, revision: inputs.instrument.revision };
}

function tagsOf(
  inputs: ModelInputs,
  { views, persistence, roll }: Parts,
  shown: Pick<GraphModel, 'persistence'>,
): GraphTag[] {
  const tags: GraphTag[] = [];
  if (inputs.view === 'fft') tags.push({ id: 'fft', corner: 'top-right', text: fftTag(inputs, views) });
  if (inputs.view === 'xy' && !views.xy.problem) {
    tags.push({ id: 'xy-y', corner: 'top-left', text: `Y  ${views.xy.scales.y}`, channel: views.xy.y });
    tags.push({ id: 'xy-x', corner: 'bottom-right', text: `X  ${views.xy.scales.x}`, channel: views.xy.x });
  }
  if (shown.persistence) tags.push({ id: 'persistence', corner: 'top-right', text: persistence.tag });
  if (roll.active) tags.push({ id: 'now', corner: 'edge-right', text: 'now' });
  return tags;
}

const fftTag = ({ spectrum }: ModelInputs, views: ViewsModel): string =>
  [
    'FFT',
    views.fft.windowLabel,
    spectrum?.scale === 'linear' ? 'V' : 'dBV',
    spectrum ? `bin ${valueText(spectrum.binSpacing)}` : '',
  ]
    .filter(Boolean)
    .join(' · ');

function describe(inputs: ModelInputs, shown: Shown, rolling: boolean): string {
  const { instrument, view } = inputs;
  const name = view === 'xy' ? 'XY' : view === 'fft' ? 'FFT' : rolling ? 'Roll' : 'Yt';
  if (shown.channels.length === 0 || !instrument) return `${name} ${SCREEN} No signal captured yet.`;
  const traces = [...shown.channels]
    .sort((a, b) => a.channel - b.channel)
    .map(({ channel }) => {
      const range = instrument.channels.find((c) => c.number === channel)?.range.label;
      return range ? `CH${channel} at ${range.text} ${range.unit}` : `CH${channel}`;
    });
  if (view !== 'yt' || rolling) return `${name} ${SCREEN} ${traces.join(', ')}.`;
  const { trigger } = instrument;
  return `${name} ${SCREEN} ${traces.join(', ')}. Trigger on CH${trigger.source}, ${trigger.slope} edge at ${trigger.level.text} ${trigger.level.unit}.`;
}

export interface Parts {
  views: ViewsModel;
  persistence: PersistenceModel;
  roll: RollModel;
}

/** What is drawn for the chosen view; the plain data of one screen, scaled to divisions by the backend. */
export function selectGraph(inputs: ModelInputs, parts: Parts): GraphModel {
  const { connection, instrument, selected, view } = inputs;
  const rolling = parts.roll.active && view === 'yt';
  const shown = drawOrder(sourcesOf(inputs, rolling), selected);
  const xy = xyTrace(inputs, parts.views);
  const hasData = view === 'xy' ? inputs.frame !== undefined : shown.channels.length > 0 || shown.math !== undefined;
  const state = graphState(connection, instrument, hasData);
  const persistence = persistenceOf(inputs, rolling);
  return {
    state,
    view,
    running: isAcquiring(connection, instrument),
    dimmed: state !== 'empty' && connection?.state !== 'ready',
    ...shown,
    ...(view === 'fft' && spectrumTicks(inputs)),
    ...(xy && { xy }),
    ...(persistence && { persistence }),
    selected,
    summary: describe(inputs, shown, rolling),
    tags: tagsOf(inputs, parts, { persistence }),
  };
}
