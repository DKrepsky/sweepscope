import { get, writable, type Writable } from 'svelte/store';
import { describe, expect, it, vi } from 'vitest';
import type { AcquisitionStatus, ConnectionSnapshot, InstrumentFacts, InstrumentState } from '../../api';
import { acquisitionStatus, connectionSnapshot, instrumentFacts, instrumentState } from '../instrument/test-data';
import type { InstrumentCommands } from '../instrument';
import { decodeFrame, type DecodedFrame } from '../waveform';
import { buildFrame } from '../waveform/test-frame';
import { createLiveMainWindow } from './live-main-window';

const value = (text: string, unit: string) => ({ value: Number(text), text, unit });

function setup(initial: { instrument?: InstrumentState; connection?: ConnectionSnapshot } = {}) {
  const connection: Writable<ConnectionSnapshot | undefined> = writable(initial.connection ?? connectionSnapshot());
  const instrument: Writable<InstrumentState | undefined> = writable(initial.instrument ?? instrumentState());
  const status: Writable<AcquisitionStatus | undefined> = writable(acquisitionStatus());
  const facts: Writable<InstrumentFacts | undefined> = writable(instrumentFacts());
  const frames: Writable<DecodedFrame | undefined> = writable(undefined);
  const commands: InstrumentCommands = {
    setRunning: vi.fn(),
    forceTrigger: vi.fn(),
    setChannel: vi.fn(),
    step: vi.fn(),
    reset: vi.fn(),
    setTriggerPosition: vi.fn(),
    setTriggerLevel: vi.fn(),
    setTrigger: vi.fn(),
    useSimulator: vi.fn(),
    retryConnection: vi.fn(),
  };
  const copyText = vi.fn();
  const notify = vi.fn();
  const window = createLiveMainWindow({
    stores: { connection, instrument, status, facts },
    commands,
    frames,
    copyText,
    notify,
    renderer: 'WebGL2',
  });
  return { window, connection, instrument, status, facts, frames, commands, copyText, notify };
}

const model = (window: ReturnType<typeof setup>['window']) => get(window);

const withChannel = (number: number, patch: Partial<InstrumentState['channels'][number]>): InstrumentState => {
  const state = instrumentState();
  return { ...state, channels: state.channels.map((c) => (c.number === number ? { ...c, ...patch } : c)) };
};

describe('live main window model: toolbar', () => {
  it('given a ready device that runs, when read, then the toolbar is running with the run chips', () => {
    const { window } = setup();
    expect(model(window).toolbar).toMatchObject({ running: true, chips: [{ kind: 'run' }, { kind: 'armed' }] });
  });

  it('given a ready device that is stopped, when read, then the toolbar is stopped with the stop chip', () => {
    const { window } = setup({ instrument: instrumentState({ running: false }) });
    expect(model(window).toolbar).toMatchObject({ running: false, chips: [{ kind: 'stop' }] });
  });

  it('given the backend disables Run, when read, then the toolbar carries the reason', () => {
    const controls = { ...instrumentState().controls, run: { enabled: false, reason: 'No scope connected' } };
    const { window } = setup({ instrument: instrumentState({ controls }) });
    expect(model(window).toolbar.availability).toEqual({ enabled: false, reason: 'No scope connected' });
  });

  it('given no instrument state yet, when read, then Run and Force are unavailable with the connection reason', () => {
    const { window, instrument, connection } = setup();
    connection.set(connectionSnapshot({ state: 'connecting' }));
    instrument.set(undefined);
    expect(model(window).toolbar).toMatchObject({
      availability: { enabled: false, reason: 'Connecting' },
      force: { enabled: false, reason: 'Connecting' },
    });
  });
});

describe('live main window model: channels', () => {
  it('given the backend state, when read, then there are four channels with formatted values', () => {
    const { window } = setup();
    const channels = model(window).channels;
    expect(channels.map((c) => c.number)).toEqual([1, 2, 3, 4]);
    expect(channels[0]).toMatchObject({
      enabled: true,
      voltsPerDiv: { value: '1', unit: 'V/div' },
      position: { value: '0', unit: 'div' },
      coupling: 'dc',
      probe: '1×',
    });
  });

  it('given a probe of 10x, when read, then the probe text uses the multiplication sign', () => {
    const { window } = setup({ instrument: withChannel(1, { probe: 10, probeText: '10x' }) });
    expect(model(window).channels[0]?.probe).toBe('10×');
  });

  it('given positions of minus 2 and plus 3 divisions, when read, then the ground markers are at 0.25 and 0.875', () => {
    const base = withChannel(1, { positionDivs: -2 });
    const state = { ...base, channels: base.channels.map((c) => (c.number === 2 ? { ...c, positionDivs: 3 } : c)) };
    const { window } = setup({ instrument: state });
    expect(
      model(window)
        .channels.map((c) => c.groundPosition)
        .slice(0, 2),
    ).toEqual([0.25, 0.875]);
  });

  it('given a position beyond the screen, when read, then the ground marker clamps to the edge', () => {
    const { window } = setup({ instrument: withChannel(1, { positionDivs: 9 }) });
    expect(model(window).channels[0]?.groundPosition).toBe(1);
  });

  it('given the step availability of the range, when read, then each direction carries its reason', () => {
    const state = withChannel(1, {
      range: {
        ...instrumentState().channels[0]!.range,
        faster: { enabled: false, reason: 'Fastest range' },
      },
    });
    const { window } = setup({ instrument: state });
    expect(model(window).channels[0]?.voltsPerDiv.step).toEqual({
      faster: { enabled: false, reason: 'Fastest range' },
      slower: { enabled: true },
    });
  });

  it('given an unverified range, when read, then the channel carries the reason text', () => {
    const { window } = setup({ instrument: withChannel(2, { unverified: 'Unverified range: above 1 V per div.' }) });
    expect(model(window).channels[1]?.unverified).toBe('Unverified range: above 1 V per div.');
  });

  it('given a channel that cannot be disabled, when read, then the enable reason is shown', () => {
    const { window } = setup({
      instrument: withChannel(1, { enable: { enabled: false, reason: 'At least one channel must stay on' } }),
    });
    expect(model(window).channels[0]?.enableReason).toBe('At least one channel must stay on');
  });

  it('given a channel that can be enabled, when read, then there is no enable reason', () => {
    const { window } = setup();
    expect(model(window).channels[0]?.enableReason).toBeUndefined();
  });

  it('given the selected channel, when another is selected, then only that channel and the graph are selected', () => {
    const { window } = setup();
    window.selectChannel(3);
    expect(model(window).channels.map((c) => c.selected)).toEqual([false, false, true, false]);
    expect(model(window).graph.selected).toBe(3);
  });

  it('given a connection that never became ready, when read, then four disabled channels with the reason are shown', () => {
    const { window, instrument, connection } = setup();
    connection.set(connectionSnapshot({ state: 'disconnected' }));
    instrument.set(undefined);
    const channels = model(window).channels;
    expect(channels).toHaveLength(4);
    expect(
      channels.every((c) => c.availability?.enabled === false && c.availability.reason === 'No scope connected'),
    ).toBe(true);
  });
});

describe('live main window model: horizontal and trigger', () => {
  it('given the backend state, when read, then time base, trigger position and window come from it', () => {
    const { window } = setup();
    expect(model(window).horizontal).toMatchObject({
      timePerDiv: { value: '200', unit: 'us/div' },
      triggerPosition: { value: '50', unit: '%' },
      window: '10.0 of 16.4 div',
      overview: { windowStart: 0.2, windowSize: 0.6, triggerTick: 0.5 },
    });
  });

  it('given facts with a rate and an interval, when read, then the rate note shows them', () => {
    const { window, facts } = setup();
    facts.set(instrumentFacts({ sampleRate: value('250', 'MS/s'), sampleInterval: value('4.00', 'ns') }));
    expect(model(window).horizontal.rateNote).toBe('Fs 250 MS/s · Ts 4.00 ns · frame mode');
  });

  it('given no facts, when read, then the rate note shows dashes', () => {
    const { window } = setup();
    expect(model(window).horizontal.rateNote).toBe('Fs - · Ts - · frame mode');
  });

  it('given a trigger level of plus 2 divisions on source 2, when read, then the marker sits at 0.75 of the screen', () => {
    const base = instrumentState();
    const { window } = setup({
      instrument: { ...base, trigger: { ...base.trigger, source: 2, levelDivisions: 2, levelTag: 'T2 ↗ 1.00 V' } },
    });
    expect(model(window).trigger).toMatchObject({
      source: 2,
      slope: 'rising',
      sweep: 'auto',
      levelPosition: 0.75,
      timePosition: 0.5,
      levelTag: 'T2 ↗ 1.00 V',
      positionTag: '50 %',
    });
  });

  it('given the default interpolation, when changed to dots, then the horizontal model shows dots', () => {
    const { window } = setup();
    expect(model(window).horizontal.interpolation).toBe('linear');
    window.setInterpolation('dots');
    expect(model(window).horizontal.interpolation).toBe('dots');
  });
});

describe('live main window model: status', () => {
  it('given a ready simulator, when read, then the connection says Connected and simulated', () => {
    const { window } = setup();
    expect(model(window).status.connection).toMatchObject({
      state: 'Connected',
      detail: 'Hantek 6254BC',
      simulated: true,
    });
  });

  it('given facts, when read, then rate, record points, capture rate and dead time come from them', () => {
    const { window, facts } = setup();
    facts.set(
      instrumentFacts({
        sampleRate: value('250', 'MS/s'),
        recordPoints: value('4096', 'pts/ch'),
        captureRate: value('18.4', '/s'),
        deadTime: value('94', '%'),
      }),
    );
    expect(model(window).status).toMatchObject({
      sampleRate: { value: '250 MS/s' },
      record: { points: '4096 pts/ch', span: '16.4 div' },
      capture: { rate: '18.4 /s', deadTime: 'dead time 94 %' },
      renderer: 'WebGL2',
    });
  });

  it('given no facts, when read, then every fact reads as a dash', () => {
    const { window } = setup();
    expect(model(window).status).toMatchObject({
      sampleRate: { value: '-' },
      record: { points: '-' },
      capture: { rate: '-', deadTime: '-' },
    });
  });
});

describe('live main window model: graph', () => {
  it('given no frame yet, when read, then the graph is empty', () => {
    const { window } = setup();
    expect(model(window).graph).toMatchObject({ state: 'empty', channels: [] });
  });

  it('given a frame while acquiring, when read, then the graph is live with the frame samples', () => {
    const { window, frames } = setup();
    frames.set(decodeFrame(buildFrame({ channels: [1, 2], samplesPerChannel: 3 })));
    const graph = model(window).graph;
    expect(graph.state).toBe('live');
    expect(graph.channels.map((c) => [c.channel, Array.from(c.samples)])).toEqual([
      [2, [2, 2, 2]],
      [1, [1, 1, 1]],
    ]);
  });

  it('given a frame and the device stops, when read, then the graph is frozen on the last frame', () => {
    const { window, frames, instrument } = setup();
    frames.set(decodeFrame(buildFrame()));
    instrument.set(instrumentState({ running: false }));
    expect(model(window).graph).toMatchObject({ state: 'frozen', channels: [{ channel: 1 }] });
  });

  it('given channel 3 is selected, when read, then it is drawn last so it lies on top', () => {
    const { window, frames } = setup();
    frames.set(decodeFrame(buildFrame({ channels: [1, 3, 4], samplesPerChannel: 2 })));
    window.selectChannel(3);
    expect(model(window).graph.channels.map((c) => c.channel)).toEqual([1, 4, 3]);
  });

  it('given a glitch of one sample, when read, then the model keeps it untouched', () => {
    const { window, frames } = setup();
    const buffer = buildFrame({ samplesPerChannel: 2501 });
    new DataView(buffer).setFloat32(48 + 4 * 1200, 7.5, true);
    frames.set(decodeFrame(buffer));
    const samples = model(window).graph.channels[0]!.samples;
    expect(samples).toHaveLength(2501);
    expect(samples[1200]).toBe(7.5);
  });

  it('given a frame, when read, then the summary names the channels and their scale', () => {
    const { window, frames } = setup();
    frames.set(decodeFrame(buildFrame()));
    expect(model(window).graph.summary).toContain('CH1');
    expect(model(window).graph.summary).toContain('1 V/div');
  });
});

describe('live main window model: banners', () => {
  it('given a ready device, when read, then there are no banners', () => {
    expect(model(setup().window).banners).toEqual([]);
  });

  it('given a connecting device, when read, then there are no banners', () => {
    const { window, connection } = setup();
    connection.set(connectionSnapshot({ state: 'connecting' }));
    expect(model(window).banners).toEqual([]);
  });

  it('given no scope found, when read, then a warning offers Retry and Use simulator', () => {
    const { window, connection } = setup();
    connection.set(connectionSnapshot({ state: 'disconnected', source: null, identity: null, simulatorActive: false }));
    expect(model(window).banners).toEqual([
      expect.objectContaining({
        kind: 'warning',
        actions: [
          { id: 'retry', label: 'Retry' },
          { id: 'use-simulator', label: 'Use simulator' },
        ],
      }),
    ]);
  });

  it('given a permission error with help, when read, then the banner offers Copy command and Retry', () => {
    const { window, connection } = setup();
    connection.set(
      connectionSnapshot({
        state: 'error',
        issue: {
          kind: 'permissionDenied',
          message: 'No permission to open the scope.',
          help: {
            ruleFile: '99.rules',
            installCommand: 'sudo cp 99.rules /etc/udev/rules.d/',
            doc: 'docs/usb-permissions.md',
          },
        },
      }),
    );
    expect(model(window).banners).toEqual([
      expect.objectContaining({
        kind: 'error',
        text: 'No permission to open the scope.',
        actions: [
          { id: 'copy-command', label: 'Copy command' },
          { id: 'retry', label: 'Retry' },
        ],
      }),
    ]);
  });

  it('given a communication error, when read, then the banner offers Retry and Use simulator', () => {
    const { window, connection } = setup();
    connection.set(
      connectionSnapshot({
        state: 'error',
        issue: { kind: 'communication', message: 'The scope stopped answering.', help: null },
      }),
    );
    expect(model(window).banners[0]?.actions.map((a) => a.id)).toEqual(['retry', 'use-simulator']);
  });
});
