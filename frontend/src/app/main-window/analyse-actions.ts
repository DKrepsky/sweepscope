import type { InstrumentCommands } from '../instrument';
import type { AnalyseCommands, PeakMarkerStore, PersistenceEpoch, ViewStore, XyAxesStore } from '../views';
import type { MainWindowActions } from './actions';
import type {
  ChannelNumber,
  MathOperationName,
  PulseCondition,
  SpectrumScaleName,
  SpectrumWindowName,
  ViewTab,
} from './types';

export interface AnalyseDeps {
  commands: AnalyseCommands;
  instrument: Pick<InstrumentCommands, 'setTrigger' | 'step' | 'reset'>;
  view: ViewStore;
  xy: XyAxesStore;
  peak: PeakMarkerStore;
  persistenceEpoch: PersistenceEpoch;
  /** Current values at the moment of the call. */
  enabledChannels: () => ChannelNumber[];
  binCount: () => number;
}

export type AnalyseActions = Pick<
  MainWindowActions,
  | 'setTriggerKind'
  | 'setPulseCondition'
  | 'commitPulseWidth'
  | 'setView'
  | 'setXyAxis'
  | 'setSpectrumWindow'
  | 'setSpectrumScale'
  | 'setPeakMode'
  | 'movePeakMarker'
  | 'placePeakMarker'
  | 'setPersistence'
  | 'clearPersistence'
  | 'armSingle'
  | 'startAutoset'
  | 'setMathEnabled'
  | 'setMathOperation'
  | 'setMathSource'
  | 'setMathPosition'
  | 'stepMathScale'
  | 'resetMathScale'
  | 'resetMathPosition'
  | 'startRecording'
  | 'stopRecording'
>;

function triggerActions({ instrument }: AnalyseDeps) {
  return {
    setTriggerKind: (kind: 'edge' | 'pulse') => instrument.setTrigger({ kind }),
    setPulseCondition: (condition: PulseCondition) => instrument.setTrigger({ condition }),
    commitPulseWidth: (widthText: string) => instrument.setTrigger({ widthText }),
  };
}

function viewActions({ view, xy, peak, commands, enabledChannels, binCount }: AnalyseDeps) {
  return {
    setView: (next: ViewTab) => {
      if (next === 'xy') xy.pickDefaults(enabledChannels());
      view.set(next);
    },
    setXyAxis: (axis: 'x' | 'y', channel: ChannelNumber) => (axis === 'x' ? xy.setX(channel) : xy.setY(channel)),
    setSpectrumWindow: (window: SpectrumWindowName) => commands.setSpectrum({ window }),
    setSpectrumScale: (scale: SpectrumScaleName) => commands.setSpectrum({ scale }),
    setPeakMode: peak.setMode,
    movePeakMarker: (bins: number) => peak.moveBy(bins, binCount()),
    placePeakMarker: (bin: number) => peak.place(bin, binCount()),
  };
}

function mathActions({ commands, instrument }: AnalyseDeps) {
  return {
    setMathEnabled: (enabled: boolean) => commands.setMath({ enabled }),
    setMathOperation: (operation: MathOperationName) => commands.setMath({ operation }),
    setMathSource: (which: 'a' | 'b', channel: ChannelNumber) =>
      commands.setMath(which === 'a' ? { sourceA: channel } : { sourceB: channel }),
    setMathPosition: (positionDivs: number) => commands.setMath({ positionDivs }),
    stepMathScale: (direction: -1 | 1) => instrument.step({ kind: 'mathScale' }, direction),
    resetMathScale: () => instrument.reset({ kind: 'mathScale' }),
    resetMathPosition: () => instrument.reset({ kind: 'mathPosition' }),
  };
}

export function createAnalyseActions(deps: AnalyseDeps): AnalyseActions {
  const { commands, persistenceEpoch } = deps;
  return {
    ...triggerActions(deps),
    ...viewActions(deps),
    ...mathActions(deps),
    setPersistence: commands.setPersistence,
    clearPersistence: persistenceEpoch.clearPersistence,
    armSingle: commands.armSingle,
    startAutoset: commands.startAutoset,
    startRecording: commands.startRecording,
    stopRecording: commands.stopRecording,
  };
}
