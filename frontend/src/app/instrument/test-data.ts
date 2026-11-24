import type {
  AcquisitionStatus,
  ConnectionSnapshot,
  DisplayState,
  InstrumentFacts,
  InstrumentState,
  MeasurementsState,
  RecordingState,
  SpectrumState,
  StateSnapshot,
} from '../../api';

const enabled = { enabled: true, reason: null };
const value = (text: string, unit: string) => ({ value: Number(text), text, unit });
const scale = (text: string, unit: string) => ({
  index: 0,
  label: value(text, unit),
  verified: true,
  faster: enabled,
  slower: enabled,
});

export const connectionSnapshot = (patch: Partial<ConnectionSnapshot> = {}): ConnectionSnapshot => ({
  revision: 1,
  state: 'ready',
  source: 'simulator',
  identity: { model: 'Hantek 6254BC', firmware: null, fpgaVersion: null, calibration: 'valid', simulated: true },
  issue: null,
  simulatorActive: true,
  ...patch,
});

const channelState = (number: number) => ({
  number,
  enabled: number === 1,
  range: scale('1', 'V/div'),
  position: value('0', 'div'),
  positionDivs: 0,
  maxPositionDivs: 4,
  coupling: 'dc' as const,
  bandwidthLimit: false,
  probe: 1,
  probeText: '1x',
  invert: false,
  unverified: null,
  enable: enabled,
});

const horizontalState = (): InstrumentState['horizontal'] => ({
  timeBase: scale('200', 'us/div'),
  triggerPosition: value('50', '%'),
  triggerPositionFraction: 0.5,
  windowText: '10.0 of 16.4 div',
  windowStartFraction: 0.2,
  windowSizeFraction: 0.6,
  triggerTickFraction: 0.5,
  roll: false,
});

const triggerState = (): InstrumentState['trigger'] => ({
  source: 1,
  slope: 'rising',
  sweep: 'auto',
  level: value('0', 'V'),
  levelDivisions: 0,
  levelTag: '0 V',
  positionTag: '50 %',
  kind: 'edge',
  pulse: null,
});

const controlsState = (): InstrumentState['controls'] => ({
  run: enabled,
  force: enabled,
  vertical: enabled,
  horizontal: enabled,
  trigger: enabled,
  files: enabled,
  single: enabled,
  autoset: enabled,
  record: enabled,
  frameViews: enabled,
});

const mathState = (): InstrumentState['math'] => ({
  enabled: false,
  operation: 'subtract',
  sourceA: 1,
  sourceB: 2,
  scale: scale('1', 'V/div'),
  unit: 'V',
  position: value('0', 'div'),
  positionDivs: 0,
  status: { kind: 'ok' },
  label: 'A−B',
});

export const instrumentState = (patch: Partial<InstrumentState> = {}): InstrumentState => ({
  revision: 1,
  running: true,
  channels: [1, 2, 3, 4].map(channelState),
  horizontal: horizontalState(),
  trigger: triggerState(),
  setupNotice: null,
  controls: controlsState(),
  math: mathState(),
  ...patch,
});

export const acquisitionStatus = (patch: Partial<AcquisitionStatus> = {}): AcquisitionStatus => ({
  revision: 1,
  phase: 'armed',
  autoset: { running: false, noSignal: false, outcome: null },
  ...patch,
});

export const instrumentFacts = (patch: Partial<InstrumentFacts> = {}): InstrumentFacts => ({
  revision: 1,
  sampleRate: null,
  sampleInterval: null,
  recordPoints: null,
  captureRate: null,
  deadTime: null,
  ...patch,
});

export const displayState = (patch: Partial<DisplayState> = {}): DisplayState => ({
  revision: 1,
  interpolation: 'sinc',
  automatic: true,
  persistence: 'off',
  ...patch,
});

export const measurementsState = (patch: Partial<MeasurementsState> = {}): MeasurementsState => ({
  revision: 1,
  channels: [],
  accuracyNote: 'Approximate',
  warnings: [],
  ...patch,
});

export const spectrumState = (patch: Partial<SpectrumState> = {}): SpectrumState => ({
  revision: 1,
  active: false,
  window: 'hann',
  scale: 'dbv',
  binSpacing: value('12.2', 'Hz'),
  span: value('125', 'MHz'),
  sampleRate: value('250', 'MS/s'),
  recordPoints: value('16384', 'pts'),
  topLabel: '+10 dBV',
  ticksX: [],
  ticksY: [],
  availability: enabled,
  ...patch,
});

export const recordingState = (patch: Partial<RecordingState> = {}): RecordingState => ({
  revision: 1,
  active: false,
  fileName: null,
  duration: value('0', 's'),
  bytesWritten: value('0', 'B'),
  failure: null,
  ...patch,
});

export const stateSnapshot = (patch: Partial<StateSnapshot> = {}): StateSnapshot => ({
  connection: connectionSnapshot(),
  instrument: instrumentState(),
  status: acquisitionStatus(),
  facts: instrumentFacts(),
  display: displayState(),
  measurements: measurementsState(),
  spectrum: spectrumState(),
  recording: recordingState(),
  ...patch,
});
