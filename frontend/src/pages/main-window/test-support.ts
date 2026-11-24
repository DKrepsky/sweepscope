import { vi, type Mock } from 'vitest';
import type { MainWindowActions } from '../../app/main-window';

export type SpyActions = { [Name in keyof MainWindowActions]: Mock<MainWindowActions[Name]> };

const NAMES = [
  'toggleRun',
  'forceTrigger',
  'setChannelEnabled',
  'stepVoltsPerDiv',
  'resetVoltsPerDiv',
  'setChannelPosition',
  'commitChannelPosition',
  'setCoupling',
  'toggleBandwidth',
  'cycleProbe',
  'toggleInvert',
  'selectChannel',
  'stepTimePerDiv',
  'resetTimePerDiv',
  'setTriggerPosition',
  'stepTriggerPosition',
  'setTriggerLevel',
  'stepTriggerLevel',
  'resetTriggerLevel',
  'setTriggerSource',
  'setTriggerSlope',
  'setTriggerSweep',
  'setInterpolation',
  'setTriggerKind',
  'setPulseCondition',
  'commitPulseWidth',
  'setView',
  'setXyAxis',
  'setSpectrumWindow',
  'setSpectrumScale',
  'setPeakMode',
  'movePeakMarker',
  'placePeakMarker',
  'setPersistence',
  'clearPersistence',
  'armSingle',
  'startAutoset',
  'setMathEnabled',
  'setMathOperation',
  'setMathSource',
  'setMathPosition',
  'stepMathScale',
  'resetMathScale',
  'resetMathPosition',
  'startRecording',
  'stopRecording',
  'toggleCursors',
  'moveCursor',
  'removeCursor',
  'clearCursors',
  'setMeasurementPinned',
  'saveSetup',
  'loadSetup',
  'exportCsv',
  'exportRaw',
  'saveScreenshot',
  'bannerAction',
  'copyText',
] as const satisfies readonly (keyof MainWindowActions)[];

/** Every action is a spy, so a test names the one call it expects and can assert that nothing else fired. */
export function spyActions(): SpyActions {
  return Object.fromEntries(NAMES.map((name) => [name, vi.fn()])) as unknown as SpyActions;
}

export const calledNames = (actions: SpyActions): string[] =>
  NAMES.filter((name) => actions[name].mock.calls.length > 0);
