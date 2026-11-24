import { writable, type Readable } from 'svelte/store';
import type { MainWindowActions } from './actions';
import { mockAnalysis, mockChannels, mockGraph, mockMeasurements, scale } from './mock-model';
import type {
  ChannelModel,
  ChannelNumber,
  ChipModel,
  MainWindowModel,
  PeakMode,
  PersistenceValue,
  ViewTab,
} from './types';

/**
 * MOCK ONLY: stands in for the backend-fed view model in the gallery and tests. Toggles and choices change local state so the
 * shell can be demonstrated; steps, resets and drags do nothing.
 */
export interface MockMainWindow extends Readable<MainWindowModel>, MainWindowActions {}

const runningChips: ChipModel[] = [
  { id: 'state', kind: 'run', text: 'Run' },
  { id: 'trigger', kind: 'triggered', text: '▲ Trig’d', filled: true },
  {
    id: 'auto',
    kind: 'auto',
    text: 'Auto',
    dim: true,
    tooltip: 'Shown while the sweep is auto-triggering without a trigger event',
  },
];

const stoppedChips: ChipModel[] = [{ id: 'state', kind: 'stop', text: 'Stop' }];

const toolbar = (): MainWindowModel['toolbar'] => ({
  running: true,
  chips: runningChips,
  single: { armed: false, availability: { enabled: true } },
  tools: { cursors: true, math: false, reference: false },
  mathAvailability: { enabled: true },
  cursorsAvailability: { enabled: true },
  files: {
    saveSetup: { enabled: true },
    loadSetup: { enabled: true },
    exportCsv: { enabled: true },
    exportRaw: { enabled: true },
    screenshot: { enabled: true },
  },
});

const horizontal = (): MainWindowModel['horizontal'] => ({
  timePerDiv: scale('200', 'µs/div'),
  triggerPosition: { value: '30', unit: '%' },
  window: '10.0 of 16.4 div',
  interpolation: 'sinc',
  rateNote: 'Fs 1.25 MS/s · Ts 800 ns · frame mode',
  overview: {
    windowStart: 0.195,
    windowSize: 0.61,
    triggerTick: 0.378,
    summary: 'Record overview: the visible window covers 10 of 16.4 divisions, trigger at division 6.2 of the record',
  },
});

const trigger = (): MainWindowModel['trigger'] => ({
  type: 'edge',
  source: 1,
  slope: 'rising',
  level: { value: '900', unit: 'mV' },
  levelPosition: 0.475,
  timePosition: 0.3,
  sweep: 'auto',
  levelTag: 'T1 ↗ 900 mV',
  positionTag: '30 %',
  accuracyNote: 'Accuracy note slot (wording TBD)',
});

const cursors = (): MainWindowModel['cursors'] => ({
  enabled: true,
  available: { enabled: true },
  source: 1,
  values: [
    { label: 'X1', value: '700 µs' },
    { label: 'Y1', value: '500 mV' },
    { label: 'X2', value: '1.10 ms' },
    { label: 'Y2', value: '1.50 V' },
    { label: 'ΔX', value: '400 µs', emphasised: true },
    { label: 'ΔY', value: '1.00 V' },
    { label: '1/ΔX', value: '2.50 kHz' },
  ],
  originNote: 'X origin: left edge of the window. Accuracy note slot (wording TBD).',
  lines: [
    { id: 'x1', axis: 'x', position: 0.35, tag: 'X1 700 µs', valueText: 'X1 700 microseconds' },
    { id: 'x2', axis: 'x', position: 0.55, tag: 'X2 1.10 ms', valueText: 'X2 1.10 milliseconds' },
    { id: 'y1', axis: 'y', position: 0.625, tag: 'Y1 0.50 V', valueText: 'Y1 0.50 volts' },
    { id: 'y2', axis: 'y', position: 0.375, tag: 'Y2 1.50 V', valueText: 'Y2 1.50 volts' },
  ],
  delta: { tag: 'ΔX 400 µs · 1/ΔX 2.50 kHz', position: 0.45 },
});

const status = (): MainWindowModel['status'] => ({
  connection: {
    state: 'Ready',
    detail: 'Hantek 6254BC · FW · FPGA',
    tooltip: 'Device state machine: disconnected, settling, ready, acquiring, error',
  },
  sampleRate: { value: '1.25 MS/s', tooltip: 'Effective sample rate for this time base and channel count' },
  record: { points: '4 096 pts/ch', span: '16.4 div' },
  capture: {
    rate: '18.4 /s',
    deadTime: 'dead time 94 %',
    tooltip: 'Frames per second the USB cycle delivers, and the share of time the scope is actually capturing',
  },
  renderer: 'WebGL2',
});

const initialModel = (): MainWindowModel => ({
  toolbar: toolbar(),
  horizontal: horizontal(),
  channels: mockChannels(),
  trigger: trigger(),
  cursors: cursors(),
  measurements: mockMeasurements(),
  status: status(),
  graph: mockGraph(),
  banners: [],
  calibration: { tag: { state: 'none', tooltip: { heading: '', lines: [] } }, dialog: undefined },
  ...mockAnalysis(),
});

function withRunning(model: MainWindowModel, running: boolean): MainWindowModel {
  return {
    ...model,
    toolbar: { ...model.toolbar, running, chips: running ? runningChips : stoppedChips },
    graph: { ...model.graph, state: running ? 'live' : 'frozen', running },
  };
}

const probes = ['1×', '10×', '100×'];

const nextProbe = (probe: string) => probes[(probes.indexOf(probe) + 1) % probes.length] ?? probes[0]!;

const withChannel = (
  model: MainWindowModel,
  number: ChannelNumber,
  change: (channel: ChannelModel) => Partial<ChannelModel>,
): MainWindowModel => ({
  ...model,
  channels: model.channels.map((channel) => (channel.number === number ? { ...channel, ...change(channel) } : channel)),
});

const withSelected = (model: MainWindowModel, number: ChannelNumber): MainWindowModel => ({
  ...model,
  channels: model.channels.map((channel) => ({ ...channel, selected: channel.number === number })),
  graph: { ...model.graph, selected: number },
});

const nothing = () => {};

const inertCalibrationActions = {
  openCalibration: nothing,
  startCalibration: nothing,
  calibrateChannel: nothing,
  continueChannel: nothing,
  closeCalibration: nothing,
};

const inertCursorAndFileActions = {
  toggleCursors: nothing,
  moveCursor: nothing,
  removeCursor: nothing,
  clearCursors: nothing,
  setMeasurementPinned: nothing,
  saveSetup: nothing,
  loadSetup: nothing,
  exportCsv: nothing,
  exportRaw: nothing,
  saveScreenshot: nothing,
};

const inertAnalysisActions = {
  setTriggerKind: nothing,
  setPulseCondition: nothing,
  commitPulseWidth: nothing,
  clearPulseWidthError: nothing,
  setSpectrumWindow: nothing,
  setSpectrumScale: nothing,
  movePeakMarker: nothing,
  placePeakMarker: nothing,
  armSingle: nothing,
  startAutoset: nothing,
  setMathOperation: nothing,
  setMathSource: nothing,
  setMathPosition: nothing,
  stepMathScale: nothing,
  resetMathScale: nothing,
  resetMathPosition: nothing,
  startRecording: nothing,
  stopRecording: nothing,
};

function analysisActions(change: (update: (current: MainWindowModel) => MainWindowModel) => void) {
  return {
    ...inertAnalysisActions,
    setView: (tab: ViewTab) => change((current) => ({ ...current, views: { ...current.views, tab } })),
    setXyAxis: (axis: 'x' | 'y', channel: ChannelNumber) =>
      change((current) => ({ ...current, views: { ...current.views, xy: { ...current.views.xy, [axis]: channel } } })),
    setPeakMode: (mode: PeakMode) => change((current) => ({ ...current, peak: { ...current.peak, mode } })),
    setPersistence: (value: PersistenceValue) =>
      change((current) => ({ ...current, persistence: { ...current.persistence, value } })),
    clearPersistence: () =>
      change((current) => ({
        ...current,
        persistence: { ...current.persistence, epoch: current.persistence.epoch + 1 },
      })),
    setMathEnabled: (enabled: boolean) =>
      change((current) => ({
        ...current,
        math: { ...current.math, row: { ...current.math.row, enabled } },
        toolbar: { ...current.toolbar, tools: { ...current.toolbar.tools, math: enabled } },
      })),
  };
}

export function createMockMainWindow(): MockMainWindow {
  const model = writable<MainWindowModel>(initialModel());
  const change = (update: (current: MainWindowModel) => MainWindowModel) => model.update(update);
  return {
    subscribe: model.subscribe,
    toggleRun: () => change((current) => withRunning(current, !current.toolbar.running)),
    forceTrigger: nothing,
    setChannelEnabled: (n, on) => change((current) => withChannel(current, n, () => ({ enabled: on }))),
    stepVoltsPerDiv: nothing,
    resetVoltsPerDiv: nothing,
    setChannelPosition: nothing,
    commitChannelPosition: nothing,
    setCoupling: (n, coupling) => change((current) => withChannel(current, n, () => ({ coupling }))),
    toggleBandwidth: (n) =>
      change((current) => withChannel(current, n, (c) => ({ bandwidthLimit: !c.bandwidthLimit }))),
    cycleProbe: (n) => change((current) => withChannel(current, n, (c) => ({ probe: nextProbe(c.probe) }))),
    toggleInvert: (n) => change((current) => withChannel(current, n, (c) => ({ inverted: !c.inverted }))),
    selectChannel: (n) => change((current) => withSelected(current, n)),
    stepTimePerDiv: nothing,
    resetTimePerDiv: nothing,
    setTriggerPosition: nothing,
    stepTriggerPosition: nothing,
    setTriggerLevel: nothing,
    stepTriggerLevel: nothing,
    resetTriggerLevel: nothing,
    setTriggerSource: (source) => change((current) => ({ ...current, trigger: { ...current.trigger, source } })),
    setTriggerSlope: (slope) => change((current) => ({ ...current, trigger: { ...current.trigger, slope } })),
    setTriggerSweep: (sweep) => change((current) => ({ ...current, trigger: { ...current.trigger, sweep } })),
    setInterpolation: (interpolation) =>
      change((current) => ({ ...current, horizontal: { ...current.horizontal, interpolation } })),
    ...inertCursorAndFileActions,
    ...analysisActions(change),
    ...inertCalibrationActions,
    bannerAction: nothing,
    copyText: nothing,
  };
}
