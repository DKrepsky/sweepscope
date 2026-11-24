import { describe, expect, it } from 'vitest';
import type { InstrumentState } from '../../api';
import { connectionSnapshot, displayState, instrumentState, spectrumState } from '../instrument/test-data';
import type { DecodedFrame } from '../waveform';
import type { SpectrumFrameInput } from '../views';
import type { ModelInputs } from './select-common';
import { selectGraph } from './select-graph';
import { selectRoll } from './select-roll';
import { selectPersistence, selectViews } from './select-views';
import { modelInputs } from './test-inputs';

const graphOf = (inputs: ModelInputs) =>
  selectGraph(inputs, {
    views: selectViews(inputs),
    persistence: selectPersistence(inputs),
    roll: selectRoll(inputs),
  });

const trace = (channel: number, value = channel) => ({ channel, samples: new Float32Array([value, value]) });
const frameOf = (...channels: number[]) => ({ channels: channels.map((c) => trace(c)) }) as unknown as DecodedFrame;

const bothOn = () =>
  instrumentState({
    channels: instrumentState().channels.map((c) => ({ ...c, enabled: c.number <= 2 })),
  });

const rollInstrument = (): InstrumentState => {
  const base = instrumentState();
  return { ...base, horizontal: { ...base.horizontal, roll: true } };
};

describe('selectGraph in Yt', () => {
  it('given channels 1, 2 and the math trace, when channel 1 is selected, then it is drawn last and math is split off', () => {
    const graph = graphOf(modelInputs({ frame: frameOf(1, 2, 5), selected: 1 }));
    expect(graph.channels.map((c) => c.channel)).toEqual([2, 1]);
    expect(graph.math?.samples).toHaveLength(2);
  });

  it('given channels 1 and 3 with channel 1 selected, when summarised, then the channels are named in numeric order', () => {
    const graph = graphOf(modelInputs({ frame: frameOf(1, 3), selected: 1 }));
    expect(graph.summary).toMatch(/CH1 at .*, CH3 at /);
  });

  it('given a running scope, when a frame is shown, then it is live, running and at full intensity', () => {
    const graph = graphOf(modelInputs({ frame: frameOf(1) }));
    expect(graph).toMatchObject({ state: 'live', running: true, dimmed: false });
  });

  it('given a stopped scope, when a frame is held, then it is not running but stays at full intensity', () => {
    const graph = graphOf(modelInputs({ frame: frameOf(1), instrument: instrumentState({ running: false }) }));
    expect(graph).toMatchObject({ running: false, dimmed: false });
  });

  it('given a disconnected scope, when a frame is held, then it is dimmed', () => {
    const graph = graphOf(
      modelInputs({ frame: frameOf(1), connection: connectionSnapshot({ state: 'disconnected' }) }),
    );
    expect(graph).toMatchObject({ state: 'frozen', dimmed: true });
  });

  it('given no frame, when read, then the graph is empty and not dimmed', () => {
    expect(graphOf(modelInputs())).toMatchObject({ state: 'empty', dimmed: false, channels: [] });
  });
});

describe('selectGraph persistence', () => {
  const persistent = (patch: Partial<ModelInputs> = {}) =>
    modelInputs({
      frame: frameOf(1),
      display: displayState({ persistence: '1s' }),
      instrument: instrumentState({ revision: 7 }),
      persistenceEpoch: 3,
      ...patch,
    });

  it('given persistence 1 s in Yt, when read, then the ghosts carry the decay, the epoch and the setup revision', () => {
    expect(graphOf(persistent()).persistence).toEqual({ decayMs: 1000, epoch: 3, revision: 7 });
  });

  it.each([
    ['500ms', 500],
    ['2s', 2000],
    ['infinite', 'infinite'],
  ] as const)('given persistence %s, when read, then the decay is %s', (value, decayMs) => {
    const graph = graphOf(persistent({ display: displayState({ persistence: value }) }));
    expect(graph.persistence?.decayMs).toBe(decayMs);
  });

  it('given persistence off, when read, then no ghosts are drawn', () => {
    expect(graphOf(persistent({ display: displayState({ persistence: 'off' }) })).persistence).toBeUndefined();
  });

  it.each(['xy', 'fft'] as const)('given the %s view, when persistence is on, then no ghosts are drawn', (view) => {
    expect(graphOf(persistent({ view })).persistence).toBeUndefined();
  });

  it('given roll mode, when persistence is on, then no ghosts are drawn', () => {
    expect(graphOf(persistent({ instrument: rollInstrument() })).persistence).toBeUndefined();
  });

  it('given persistence on, when read, then the screen tag shows the setting', () => {
    expect(graphOf(persistent()).tags).toContainEqual({ id: 'persistence', corner: 'top-right', text: 'PERSIST 1 s' });
  });
});

describe('selectGraph in XY', () => {
  const xyInputs = (patch: Partial<ModelInputs> = {}) =>
    modelInputs({ view: 'xy', instrument: bothOn(), frame: frameOf(1, 2), ...patch });

  it('given two channels on, when read, then the pair is the X and Y samples and no Yt trace is drawn', () => {
    const graph = graphOf(xyInputs());
    expect(graph.xy).toMatchObject({ xChannel: 1, yChannel: 2 });
    expect(Array.from(graph.xy!.x)).toEqual([1, 1]);
    expect(Array.from(graph.xy!.y)).toEqual([2, 2]);
    expect(graph.channels).toEqual([]);
  });

  it('given XY, when read, then the axis scales are tagged in the corners', () => {
    const tags = graphOf(xyInputs()).tags;
    expect(tags).toContainEqual({ id: 'xy-y', corner: 'top-left', text: 'Y  CH2 1 V/div', channel: 2 });
    expect(tags).toContainEqual({ id: 'xy-x', corner: 'bottom-right', text: 'X  CH1 1 V/div', channel: 1 });
  });

  it('given the Y channel is off, when read, then nothing is drawn and no axis tag is shown', () => {
    const graph = graphOf(xyInputs({ xyAxes: { x: 1, y: 3 } }));
    expect(graph.xy).toBeUndefined();
    expect(graph.tags).toEqual([]);
  });
});

describe('selectGraph in FFT', () => {
  const spectrumFrame: SpectrumFrameInput = {
    bins: 3,
    binHz: 10,
    unitsPerDiv: 10,
    channels: [
      { channel: 1, peakBin: 1, levels: new Float32Array([-60, -20, -50]), divisions: new Float32Array([7, 3, 6]) },
    ],
  };

  it('given a spectrum, when read, then the traces are the level divisions', () => {
    const graph = graphOf(modelInputs({ view: 'fft', spectrumFrame }));
    expect(Array.from(graph.channels[0]!.samples)).toEqual([7, 3, 6]);
  });

  it('given backend ticks counted from the top, when read, then the level ticks count from the bottom', () => {
    const spectrum = spectrumState({
      ticksX: [{ at: 0.5, text: '25 kHz' }],
      ticksY: [
        { at: 0, text: '+10 dBV' },
        { at: 0.25, text: '-10' },
      ],
    });
    const graph = graphOf(modelInputs({ view: 'fft', spectrumFrame, spectrum }));
    expect(graph.ticksX).toEqual([{ at: 0.5, text: '25 kHz' }]);
    expect(graph.ticksY).toEqual([
      { at: 1, text: '+10 dBV' },
      { at: 0.75, text: '-10' },
    ]);
  });

  it('given a spectrum, when read, then the corner tag names the window, the unit and the bin spacing', () => {
    const graph = graphOf(modelInputs({ view: 'fft', spectrumFrame }));
    expect(graph.tags).toContainEqual({ id: 'fft', corner: 'top-right', text: 'FFT · Hann · dBV · bin 12.2 Hz' });
  });

  it('given the linear scale, when read, then the tag shows volts', () => {
    const spectrum = spectrumState({ scale: 'linear' });
    expect(graphOf(modelInputs({ view: 'fft', spectrumFrame, spectrum })).tags[0]?.text).toContain('· V ·');
  });
});

describe('selectGraph in roll', () => {
  it('given roll mode and a window, when read, then the window traces are drawn with a now mark', () => {
    const roll = { channels: [trace(1)], gaps: [], gapCount: 0 };
    const graph = graphOf(modelInputs({ instrument: rollInstrument(), roll }));
    expect(graph.channels).toHaveLength(1);
    expect(graph.tags).toContainEqual({ id: 'now', corner: 'edge-right', text: 'now' });
  });
});
