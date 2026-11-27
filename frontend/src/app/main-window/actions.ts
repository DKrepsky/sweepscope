import type {
  BannerActionId,
  ChannelNumber,
  CursorId,
  Interpolation,
  MathOperationName,
  PeakMode,
  PersistenceValue,
  PulseCondition,
  SpectrumScaleName,
  SpectrumWindowName,
  ViewTab,
} from './types';

/** Plain callbacks the pages call; they hold no store access. */
export interface MainWindowActions {
  toggleRun(): void;
  forceTrigger(): void;
  setChannelEnabled(channel: ChannelNumber, on: boolean): void;
  stepVoltsPerDiv(channel: ChannelNumber, direction: -1 | 1): void;
  resetVoltsPerDiv(channel: ChannelNumber): void;
  setChannelPosition(channel: ChannelNumber, divisions: number): void;
  commitChannelPosition(channel: ChannelNumber, text: string): void;
  setCoupling(channel: ChannelNumber, coupling: 'ac' | 'dc'): void;
  toggleBandwidth(channel: ChannelNumber): void;
  cycleProbe(channel: ChannelNumber): void;
  toggleInvert(channel: ChannelNumber): void;
  selectChannel(channel: ChannelNumber): void;
  stepTimePerDiv(direction: -1 | 1): void;
  resetTimePerDiv(): void;
  setTriggerPosition(fraction: number): void;
  stepTriggerPosition(direction: -1 | 1): void;
  /** Divisions from the screen centre, up is positive. */
  setTriggerLevel(divisionsFromCentre: number): void;
  stepTriggerLevel(direction: -1 | 1): void;
  resetTriggerLevel(): void;
  setTriggerSource(channel: ChannelNumber): void;
  setTriggerSlope(slope: 'rising' | 'falling'): void;
  setTriggerSweep(sweep: 'auto' | 'normal'): void;
  setInterpolation(interpolation: Interpolation): void;
  setTriggerKind(kind: 'edge' | 'pulse'): void;
  setPulseCondition(condition: PulseCondition): void;
  /** Text as typed (`500 us`); the backend parses it and rejects the whole patch when it is not a time. */
  commitPulseWidth(text: string): void;
  /** Escape or blur of the width field: the shown error goes away. */
  clearPulseWidthError(): void;
  setView(view: ViewTab): void;
  setXyAxis(axis: 'x' | 'y', channel: ChannelNumber): void;
  setSpectrumWindow(window: SpectrumWindowName): void;
  setSpectrumScale(scale: SpectrumScaleName): void;
  setPeakMode(mode: PeakMode): void;
  /** Bins from the current marker; the marker becomes manual. */
  movePeakMarker(bins: number): void;
  placePeakMarker(bin: number): void;
  setPersistence(value: PersistenceValue): void;
  clearPersistence(): void;
  armSingle(): void;
  startAutoset(): void;
  setMathEnabled(enabled: boolean): void;
  setMathOperation(operation: MathOperationName): void;
  setMathSource(which: 'a' | 'b', channel: ChannelNumber): void;
  setMathPosition(divisions: number): void;
  stepMathScale(direction: -1 | 1): void;
  resetMathScale(): void;
  resetMathPosition(): void;
  startRecording(): void;
  stopRecording(): void;
  toggleCursors(): void;
  moveCursor(id: CursorId, position: number): void;
  removeCursor(id: CursorId): void;
  clearCursors(): void;
  setMeasurementPinned(id: string, pinned: boolean): void;
  saveSetup(): void;
  loadSetup(): void;
  exportCsv(): void;
  exportRaw(): void;
  saveScreenshot(): void;
  /** Refused by the guard while calibration is unavailable; the tag is disabled then too. */
  openCalibration(): void;
  startCalibration(): void;
  calibrateChannel(): void;
  continueChannel(): void;
  closeCalibration(): void;
  bannerAction(id: BannerActionId): void;
  copyText(text: string): void;
}
