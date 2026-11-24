import { get, writable, type Writable } from 'svelte/store';
import { describe, expect, it, vi } from 'vitest';
import type {
  AcquisitionStatus,
  ConnectionSnapshot,
  DisplayState,
  InstrumentFacts,
  InstrumentState,
  MeasurementsState,
} from '../../api';
import { createCursorsStore } from '../cursors';
import {
  acquisitionStatus,
  connectionSnapshot,
  instrumentFacts,
  instrumentState,
  measurementsState,
} from '../instrument/test-data';
import type { InstrumentCommands } from '../instrument';
import type { AnalyseCommands, RollInput, SpectrumFrameInput } from '../views';
import type { RecordingState } from '../../api/generated/RecordingState';
import type { SpectrumState } from '../../api/generated/SpectrumState';
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
    setInterpolation: vi.fn(),
    armSingle: vi.fn(),
    startAutoset: vi.fn(),
    setMath: vi.fn(),
    setPersistence: vi.fn(),
    setSpectrum: vi.fn(),
    startRecording: vi.fn(),
    stopRecording: vi.fn(),
  };
  const display: Writable<DisplayState | undefined> = writable(undefined);
  const measurements: Writable<MeasurementsState | undefined> = writable(undefined);
  const cursors = createCursorsStore();
  const pinned = { ...writable<readonly string[]>([]), set: vi.fn(() => Promise.resolve()), load: vi.fn() };
  const files = {
    saveSetup: vi.fn(() => Promise.resolve()),
    loadSetup: vi.fn(() => Promise.resolve()),
    exportCsv: vi.fn(() => Promise.resolve()),
    exportRaw: vi.fn(() => Promise.resolve()),
    saveScreenshot: vi.fn(() => Promise.resolve()),
  };
  const analyseCommands: AnalyseCommands = {
    armSingle: vi.fn(),
    startAutoset: vi.fn(),
    setMath: vi.fn(),
    setPersistence: vi.fn(),
    setSpectrum: vi.fn(),
    startRecording: vi.fn(),
    stopRecording: vi.fn(),
  };
  const spectrum: Writable<SpectrumState | undefined> = writable(undefined);
  const spectrumFrame: Writable<SpectrumFrameInput | undefined> = writable(undefined);
  const recording: Writable<RecordingState | undefined> = writable(undefined);
  const roll: Writable<RollInput | undefined> = writable(undefined);
  const copyText = vi.fn();
  const notify = vi.fn();
  const window = createLiveMainWindow({
    stores: { connection, instrument, status, facts, display, measurements },
    analysis: { spectrum, spectrumFrame, recording, roll },
    analyseCommands,
    commands,
    cursors,
    pinned,
    files,
    frames,
    copyText,
    notify,
    renderer: 'WebGL2',
  });
  return {
    window,
    connection,
    instrument,
    status,
    facts,
    display,
    measurements,
    cursors,
    frames,
    commands,
    analyseCommands,
    spectrumFrame,
    copyText,
    notify,
  };
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

  it('given a verified time base, when read, then the horizontal model carries no unverified marker', () => {
    const { window } = setup();
    expect(model(window).horizontal.unverified).toBeUndefined();
  });

  it('given an unverified time base, when read, then the horizontal model carries the unverified marker text', () => {
    const state = instrumentState();
    const timeBase = { ...state.horizontal.timeBase, verified: false };
    const { window } = setup({ instrument: { ...state, horizontal: { ...state.horizontal, timeBase } } });
    expect(model(window).horizontal.unverified).toBe('Unverified (see accuracy note).');
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

  it('given the backend display state, when it changes, then the horizontal model follows it', () => {
    const { window, display } = setup();
    expect(model(window).horizontal.interpolation).toBe('linear');
    display.set({ revision: 1, interpolation: 'sinc', automatic: true, persistence: 'off' });
    expect(model(window).horizontal.interpolation).toBe('sinc');
  });

  it('given the backend display state, when automatic changes, then the horizontal model reports it', () => {
    const { window, display } = setup();
    display.set({ revision: 1, interpolation: 'sinc', automatic: true, persistence: 'off' });
    expect(model(window).horizontal.automaticInterpolation).toBe(true);
    display.set({ revision: 2, interpolation: 'dots', automatic: false, persistence: 'off' });
    expect(model(window).horizontal.automaticInterpolation).toBe(false);
  });

  it('given an interpolation choice, when made, then it goes to the backend and the model keeps the backend value', () => {
    const { window, commands } = setup();
    window.setInterpolation('dots');
    expect(commands.setInterpolation).toHaveBeenCalledExactlyOnceWith('dots');
    expect(model(window).horizontal.interpolation).toBe('linear');
  });
});

describe('live main window model: cursors, files and statistics', () => {
  it('given a waveform and no cursors, when toggled from the window, then the toolbar shows the cursors tool as on', () => {
    const { window, frames } = setup();
    frames.set(decodeFrame(buildFrame()));
    window.toggleCursors();
    expect(model(window).toolbar.tools.cursors).toBe(true);
  });

  it('given no waveform, when cursors are toggled from the window, then no cursor is added', () => {
    const { window } = setup();
    window.toggleCursors();
    expect(model(window).cursors.lines).toEqual([]);
  });

  it('given no waveform, when read, then the toolbar cursors tool is unavailable with the reason', () => {
    const { window } = setup();
    expect(model(window).toolbar.cursorsAvailability).toEqual({ enabled: false, reason: 'No waveform' });
  });

  it('given no waveform yet, when read, then cursors are unavailable with the reason', () => {
    const { window } = setup();
    expect(model(window).cursors.available).toEqual({ enabled: false, reason: 'No waveform' });
  });

  it('given no waveform, when read, then exports and screenshot are disabled with the reason', () => {
    const { window } = setup();
    const { files } = model(window).toolbar;
    expect([files.exportCsv, files.exportRaw, files.screenshot]).toEqual(
      Array(3).fill({ enabled: false, reason: 'No waveform' }),
    );
  });

  it('given a waveform, when read, then exports and screenshot are enabled', () => {
    const { window, frames } = setup();
    frames.set(decodeFrame(buildFrame({ sequence: 1 })));
    const { files } = model(window).toolbar;
    expect([files.exportCsv, files.exportRaw, files.screenshot]).toEqual(Array(3).fill({ enabled: true }));
  });

  it('given the backend disables setup files, when read, then both setup items carry its reason', () => {
    const controls = { ...instrumentState().controls, files: { enabled: false, reason: 'No scope connected' } };
    const { window } = setup({ instrument: instrumentState({ controls }) });
    const { files } = model(window).toolbar;
    expect([files.saveSetup, files.loadSetup]).toEqual(Array(2).fill({ enabled: false, reason: 'No scope connected' }));
  });

  it('given the live window, when read, then statistics are hidden', () => {
    const { window } = setup();
    expect(model(window).measurements.statistics).toBeUndefined();
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

describe('live main window model: status accuracy note', () => {
  it('given no measurements slice yet, when read, then the status accuracy says there is no measurement yet', () => {
    const { window } = setup();
    expect(model(window).status.accuracy).toBe('No measurement yet');
  });

  it('given a measurements slice, when read, then the status accuracy is the slice note', () => {
    const { window, measurements } = setup();
    measurements.set(measurementsState({ accuracyNote: 'Backend wording.' }));
    expect(model(window).status.accuracy).toBe('Backend wording.');
  });
});

describe('live main window model: analysis actions', () => {
  it('given the Yt view, when FFT is chosen and then Yt again, then the backend is told active true and then false', () => {
    const { window, analyseCommands } = setup();

    window.setView('fft');
    window.setView('yt');

    expect(vi.mocked(analyseCommands.setSpectrum).mock.calls).toEqual([[{ active: true }], [{ active: false }]]);
  });

  it('given the Yt view, when XY and then FFT are chosen, then the model follows and the backend hears only FFT', () => {
    const { window, analyseCommands } = setup();

    window.setView('xy');
    expect(model(window).views.tab).toBe('xy');
    window.setView('fft');

    expect(model(window).views.tab).toBe('fft');
    expect(analyseCommands.setSpectrum).toHaveBeenCalledExactlyOnceWith({ active: true });
  });

  it('given channels 3 and 4 enabled, when XY is entered, then the axes are those two channels', () => {
    const base = instrumentState();
    const only = (numbers: number[]): InstrumentState => ({
      ...base,
      channels: base.channels.map((c) => ({ ...c, enabled: numbers.includes(c.number) })),
    });
    const { window } = setup({ instrument: only([3, 4]) });

    window.setView('xy');

    expect(model(window).views.xy).toMatchObject({ x: 3, y: 4 });
  });

  it('given X is CH1, when Y is set to CH1, then it is refused', () => {
    const { window } = setup();

    window.setXyAxis('y', 1);

    expect(model(window).views.xy).toMatchObject({ x: 1, y: 2 });
  });

  it('given the persistence menu, when a value is chosen, then it goes to the backend and the model keeps the backend value', () => {
    const { window, analyseCommands } = setup();

    window.setPersistence('1s');

    expect(analyseCommands.setPersistence).toHaveBeenCalledExactlyOnceWith('1s');
    expect(model(window).persistence.value).toBe('off');
  });

  it('given a backend persistence value, when read, then the model shows it and its tag', () => {
    const { window, display } = setup();

    display.set({ revision: 1, interpolation: 'sinc', automatic: true, persistence: '500ms' });

    expect(model(window).persistence).toMatchObject({ value: '500ms', tag: 'PERSIST 0.5 s' });
  });

  it('given persistence, when cleared, then the epoch changes and no command is sent', () => {
    const { window, analyseCommands } = setup();
    const before = model(window).persistence.epoch;

    window.clearPersistence();

    expect(model(window).persistence.epoch).toBe(before + 1);
    expect(analyseCommands.setPersistence).not.toHaveBeenCalled();
  });

  it('given the XY view, when read, then persistence is unavailable with the Yt-only reason', () => {
    const { window } = setup();

    window.setView('xy');

    expect(model(window).persistence.available).toEqual({
      enabled: false,
      reason: 'Persistence is available in Yt only',
    });
  });

  it('given single, autoset and recording requests, when made, then each reaches its command', () => {
    const { window, analyseCommands } = setup();

    window.armSingle();
    window.startAutoset();
    window.startRecording();
    window.stopRecording();

    expect(analyseCommands.armSingle).toHaveBeenCalledOnce();
    expect(analyseCommands.startAutoset).toHaveBeenCalledOnce();
    expect(analyseCommands.startRecording).toHaveBeenCalledOnce();
    expect(analyseCommands.stopRecording).toHaveBeenCalledOnce();
  });

  it('given math and spectrum choices, when made, then each is one patch with only that field', () => {
    const { window, analyseCommands } = setup();

    window.setMathEnabled(true);
    window.setMathOperation('add');
    window.setMathSource('a', 3);
    window.setMathSource('b', 4);
    window.setMathPosition(1.5);
    window.setSpectrumWindow('blackman');
    window.setSpectrumScale('linear');

    expect(vi.mocked(analyseCommands.setMath).mock.calls).toEqual([
      [{ enabled: true }],
      [{ operation: 'add' }],
      [{ sourceA: 3 }],
      [{ sourceB: 4 }],
      [{ positionDivs: 1.5 }],
    ]);
    expect(vi.mocked(analyseCommands.setSpectrum).mock.calls).toEqual([
      [{ window: 'blackman' }],
      [{ scale: 'linear' }],
    ]);
  });

  it('given math scale and position, when stepped and reset, then the instrument targets are used', () => {
    const { window, commands } = setup();

    window.stepMathScale(-1);
    window.resetMathScale();
    window.resetMathPosition();

    expect(commands.step).toHaveBeenCalledExactlyOnceWith({ kind: 'mathScale' }, -1);
    expect(vi.mocked(commands.reset).mock.calls).toEqual([[{ kind: 'mathScale' }], [{ kind: 'mathPosition' }]]);
  });

  it('given the trigger, when kind, condition and width are chosen, then each is a trigger patch', () => {
    const { window, commands } = setup();

    window.setTriggerKind('pulse');
    window.setPulseCondition('greater');
    window.commitPulseWidth('500 us');

    expect(vi.mocked(commands.setTrigger).mock.calls).toEqual([
      [{ kind: 'pulse' }],
      [{ condition: 'greater' }],
      [{ widthText: '500 us' }, expect.any(Function)],
    ]);
  });

  it('given a spectrum with 100 bins, when the peak marker is placed and moved past the end, then it stays on the last bin in manual mode', () => {
    const { window, spectrumFrame } = setup();
    spectrumFrame.set({ bins: 100, binHz: 12.21, channels: [] });

    window.placePeakMarker(97);
    window.movePeakMarker(10);

    expect(model(window).peak).toMatchObject({ mode: 'manual', bin: 99 });
  });

  it('given a channel is selected, when read, then the peak marker follows it', () => {
    const { window } = setup();

    window.selectChannel(3);

    expect(model(window).peak.channel).toBe(3);
  });

  it('given no analysis feeds, when read, then roll, recording, math and peak are disabled or empty', () => {
    const { window } = setup();

    expect(model(window)).toMatchObject({
      roll: { active: false, gaps: [], gapCount: 0 },
      recording: { active: false },
      math: { row: { enabled: false }, chip: '' },
      autoset: { busy: false, noSignal: false },
    });
  });
});

describe('live main window model: pulse trigger and math', () => {
  const pulseState = (): InstrumentState => {
    const base = instrumentState();
    return {
      ...base,
      trigger: {
        ...base.trigger,
        kind: 'pulse',
        pulse: { condition: 'greater', width: value('500', 'µs') as never, verified: true },
      },
    };
  };

  it('given a pulse trigger, when read, then the type, condition and applied width show', () => {
    const { window } = setup({ instrument: pulseState() });

    expect(model(window).trigger).toMatchObject({
      type: 'pulse',
      pulse: { condition: 'greater', width: { value: '500', unit: 'µs' }, verified: true },
    });
    expect(model(window).trigger.pulse?.error).toBeUndefined();
  });

  it('given a typed width, when the backend rejects it, then the model carries the error with the applied width', () => {
    const { window, commands } = setup({ instrument: pulseState() });
    vi.mocked(commands.setTrigger).mockImplementation((_patch, onRejected) => onRejected?.());

    window.commitPulseWidth('abc');

    expect(model(window).trigger.pulse?.error).toBe('Invalid width. Enter a time such as 500 µs. Applied: 500 µs.');
  });

  it('given a shown width error, when Escape clears it, then the error is gone', () => {
    const { window, commands } = setup({ instrument: pulseState() });
    vi.mocked(commands.setTrigger).mockImplementation((_patch, onRejected) => onRejected?.());
    window.commitPulseWidth('abc');

    window.clearPulseWidthError();

    expect(model(window).trigger.pulse?.error).toBeUndefined();
  });

  it('given a shown width error, when a new width is committed and accepted, then the error is gone', () => {
    const { window, commands } = setup({ instrument: pulseState() });
    vi.mocked(commands.setTrigger).mockImplementationOnce((_patch, onRejected) => onRejected?.());
    window.commitPulseWidth('abc');

    window.commitPulseWidth('1 ms');

    expect(model(window).trigger.pulse?.error).toBeUndefined();
  });

  it('given an edge trigger, when read, then there is no pulse model', () => {
    const { window } = setup();

    expect(model(window).trigger).toMatchObject({ type: 'edge' });
    expect(model(window).trigger.pulse).toBeUndefined();
  });

  const withMath = (patch: Partial<InstrumentState['math']>): InstrumentState => {
    const base = instrumentState();
    return { ...base, math: { ...base.math, ...patch } };
  };

  it('given math on with sources of one scale, when read, then the row shows the label and the chip the scale', () => {
    const { window } = setup({ instrument: withMath({ enabled: true }) });

    expect(model(window).math.row).toMatchObject({ enabled: true, label: 'A−B', sourceA: 1, sourceB: 2, status: 'ok' });
    expect(model(window).math.chip).toBe('M  A−B  1 V/div');
  });

  it('given math off, when read, then the chip is empty', () => {
    const { window } = setup();

    expect(model(window).math.chip).toBe('');
  });

  it('given sources of different scales, when read, then the row is mixed with the backend detail', () => {
    const { window } = setup({
      instrument: withMath({ enabled: true, status: { kind: 'mixed', detail: 'CH1 500 mV/div, CH2 20 mV/div.' } }),
    });

    expect(model(window).math.row).toMatchObject({ status: 'mixed', statusDetail: 'CH1 500 mV/div, CH2 20 mV/div.' });
  });

  it('given a disabled source, when read, then the chip names the reason and the model carries it', () => {
    const { window } = setup({
      instrument: withMath({ enabled: true, status: { kind: 'unavailable', reason: 'CH2 is off' } }),
    });

    expect(model(window).math.chip).toBe('M  A−B  unavailable: CH2 is off');
    expect(model(window).math.unavailableReason).toBe('CH2 is off');
  });

  it('given the invert operation, when read, then there is no source B', () => {
    const { window } = setup({ instrument: withMath({ operation: 'invertA', sourceB: null }) });

    expect(model(window).math.row.sourceB).toBeUndefined();
  });
});

describe('live main window model: pulse trigger and math', () => {
  const pulseState = (): InstrumentState => {
    const base = instrumentState();
    return {
      ...base,
      trigger: {
        ...base.trigger,
        kind: 'pulse',
        pulse: { condition: 'greater', width: value('500', 'µs'), verified: true },
      },
    };
  };
  const rejecting = (commands: InstrumentCommands) =>
    vi.mocked(commands.setTrigger).mockImplementation((_patch, onRejected) => onRejected?.());

  it('given a pulse trigger, when read, then the type, condition and applied width show', () => {
    const { window } = setup({ instrument: pulseState() });

    expect(model(window).trigger).toMatchObject({
      type: 'pulse',
      pulse: { condition: 'greater', width: { value: '500', unit: 'µs' }, verified: true },
    });
    expect(model(window).trigger.pulse?.error).toBeUndefined();
  });

  it('given a typed width, when the backend rejects it, then the model carries the error with the applied width', () => {
    const { window, commands } = setup({ instrument: pulseState() });
    rejecting(commands);

    window.commitPulseWidth('abc');

    expect(model(window).trigger.pulse?.error).toBe('Invalid width. Enter a time such as 500 µs. Applied: 500 µs.');
  });

  it('given a shown width error, when Escape clears it, then the error is gone', () => {
    const { window, commands } = setup({ instrument: pulseState() });
    rejecting(commands);
    window.commitPulseWidth('abc');

    window.clearPulseWidthError();

    expect(model(window).trigger.pulse?.error).toBeUndefined();
  });

  it('given a shown width error, when a new width is committed and accepted, then the error is gone', () => {
    const { window, commands } = setup({ instrument: pulseState() });
    vi.mocked(commands.setTrigger).mockImplementationOnce((_patch, onRejected) => onRejected?.());
    window.commitPulseWidth('abc');

    window.commitPulseWidth('1 ms');

    expect(model(window).trigger.pulse?.error).toBeUndefined();
  });

  it('given an edge trigger, when read, then there is no pulse model', () => {
    const { window } = setup();

    expect(model(window).trigger).toMatchObject({ type: 'edge' });
    expect(model(window).trigger.pulse).toBeUndefined();
  });

  const withMath = (patch: Partial<InstrumentState['math']>): InstrumentState => {
    const base = instrumentState();
    return { ...base, math: { ...base.math, ...patch } };
  };

  it('given math on with sources of one scale, when read, then the row shows the label and the chip the scale', () => {
    const { window } = setup({ instrument: withMath({ enabled: true }) });

    expect(model(window).math.row).toMatchObject({ enabled: true, label: 'A−B', sourceA: 1, sourceB: 2, status: 'ok' });
    expect(model(window).math.chip).toBe('M  A−B  1 V/div');
  });

  it('given math off, when read, then the chip is empty', () => {
    const { window } = setup();

    expect(model(window).math.chip).toBe('');
  });

  it('given sources of different scales, when read, then the row is mixed with the backend detail', () => {
    const { window } = setup({
      instrument: withMath({ enabled: true, status: { kind: 'mixed', detail: 'CH1 500 mV/div, CH2 20 mV/div.' } }),
    });

    expect(model(window).math.row).toMatchObject({ status: 'mixed', statusDetail: 'CH1 500 mV/div, CH2 20 mV/div.' });
  });

  it('given a disabled source, when read, then the chip names the reason and the model carries it', () => {
    const { window } = setup({
      instrument: withMath({ enabled: true, status: { kind: 'unavailable', reason: 'CH2 is off' } }),
    });

    expect(model(window).math.chip).toBe('M  A−B  unavailable: CH2 is off');
    expect(model(window).math.unavailableReason).toBe('CH2 is off');
  });

  it('given the invert operation, when read, then there is no source B', () => {
    const { window } = setup({ instrument: withMath({ operation: 'invertA', sourceB: null }) });

    expect(model(window).math.row.sourceB).toBeUndefined();
  });
});
