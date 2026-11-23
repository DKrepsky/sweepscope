import type { BannerActionId, ChannelNumber, CursorId, Interpolation } from './types';

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
  bannerAction(id: BannerActionId): void;
  copyText(text: string): void;
}
