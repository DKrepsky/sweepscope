import type { AcquisitionStatus, ConnectionSnapshot, InstrumentFacts, InstrumentState, StateSnapshot } from '../../api';

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

export const instrumentState = (patch: Partial<InstrumentState> = {}): InstrumentState => ({
  revision: 1,
  running: true,
  channels: [1, 2, 3, 4].map((number) => ({
    number,
    enabled: number === 1,
    range: scale('1', 'V/div'),
    position: value('0', 'div'),
    positionDivs: 0,
    maxPositionDivs: 4,
    coupling: 'dc',
    bandwidthLimit: false,
    probe: 1,
    probeText: '1x',
    invert: false,
    unverified: null,
    enable: enabled,
  })),
  horizontal: {
    timeBase: scale('200', 'us/div'),
    triggerPosition: value('50', '%'),
    triggerPositionFraction: 0.5,
    windowText: '10.0 of 16.4 div',
    windowStartFraction: 0.2,
    windowSizeFraction: 0.6,
    triggerTickFraction: 0.5,
  },
  trigger: {
    source: 1,
    slope: 'rising',
    sweep: 'auto',
    level: value('0', 'V'),
    levelDivisions: 0,
    levelTag: '0 V',
    positionTag: '50 %',
  },
  controls: { run: enabled, force: enabled, vertical: enabled, horizontal: enabled, trigger: enabled },
  ...patch,
});

export const acquisitionStatus = (patch: Partial<AcquisitionStatus> = {}): AcquisitionStatus => ({
  revision: 1,
  phase: 'armed',
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

export const stateSnapshot = (patch: Partial<StateSnapshot> = {}): StateSnapshot => ({
  connection: connectionSnapshot(),
  instrument: instrumentState(),
  status: acquisitionStatus(),
  facts: instrumentFacts(),
  ...patch,
});
