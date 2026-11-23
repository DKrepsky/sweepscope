/** View model of the main window. Strings are already formatted (units, signs, precision); the backend will feed it. */

import type { CursorId } from '../cursors';

export type { CursorId };

export type ChannelNumber = 1 | 2 | 3 | 4;

export interface Quantity {
  value: string;
  unit: string;
}

/** Whether a control can be used now; `reason` is shown as its tooltip when it cannot. */
export interface Availability {
  enabled: boolean;
  reason?: string;
}

/** A stepped setting (volts or time per division): the value plus whether each direction can still be stepped. */
export interface ScaleQuantity extends Quantity {
  step: { faster: Availability; slower: Availability };
}

export type ChipKind = 'run' | 'stop' | 'armed' | 'triggered' | 'auto' | 'disconnected' | 'error' | 'offline';

export interface ChipModel {
  id: string;
  kind: ChipKind;
  text: string;
  filled?: boolean;
  dim?: boolean;
  tooltip?: string;
}

export type ViewTab = 'yt' | 'xy' | 'fft';

export interface FilesAvailability {
  saveSetup: Availability;
  loadSetup: Availability;
  exportCsv: Availability;
  exportRaw: Availability;
  screenshot: Availability;
}

export interface ToolbarModel {
  running: boolean;
  chips: ChipModel[];
  persistence: string;
  view: ViewTab;
  tools: { cursors: boolean; math: boolean; reference: boolean };
  files: FilesAvailability;
  /** Run and Stop. */
  availability?: Availability;
  force?: Availability;
}

export type Interpolation = 'sinc' | 'linear' | 'dots';

export interface HorizontalModel {
  timePerDiv: ScaleQuantity;
  /** Marker text when the selected time base step has not been measured on the hardware. */
  unverified?: string;
  triggerPosition: Quantity;
  window: string;
  interpolation: Interpolation;
  availability?: Availability;
  /** Effective sample rate line of the Horizontal group. */
  rateNote: string;
  overview: { windowStart: number; windowSize: number; triggerTick: number; summary: string };
}

export interface ChannelModel {
  number: ChannelNumber;
  enabled: boolean;
  selected: boolean;
  voltsPerDiv: ScaleQuantity;
  position: Quantity;
  coupling: 'ac' | 'dc';
  bandwidthLimit: boolean;
  probe: string;
  inverted: boolean;
  /** Reason text when the range is outside the verified one. */
  unverified?: string;
  /** Ground marker, 0 (bottom) to 1 (top) of the screen. */
  groundPosition: number;
  enableReason?: string;
  availability?: Availability;
  /** A sample of the newest record sits on a converter rail. */
  clipped?: boolean;
}

export interface TriggerModel {
  type: 'edge' | 'pulse' | 'video';
  source: ChannelNumber;
  slope: 'rising' | 'falling';
  level: Quantity;
  /** Level marker, 0 (bottom) to 1 (top) of the screen. */
  levelPosition: number;
  /** Position marker, 0 (left) to 1 (right) of the screen. */
  timePosition: number;
  sweep: 'auto' | 'normal';
  levelTag: string;
  positionTag: string;
  accuracyNote: string;
  availability?: Availability;
}

export interface CursorValue {
  label: string;
  value: string;
  emphasised?: boolean;
}

export interface CursorLineModel {
  id: CursorId;
  axis: 'x' | 'y';
  /** Fraction of the screen: x from the left, y from the top. */
  position: number;
  tag: string;
  /** Spoken readout. */
  valueText: string;
}

export interface CursorsModel {
  enabled: boolean;
  /** Why the cursors cannot be switched on (no waveform). */
  available: Availability;
  source: ChannelNumber;
  values: CursorValue[];
  originNote: string;
  lines: CursorLineModel[];
  delta?: { tag: string; position: number };
}

export interface MeasurementCellModel {
  state: 'valid' | 'unverified' | 'invalid' | 'stale';
  text: string;
  unit?: string;
  footnote?: number;
  tooltip?: string;
}

/** Columns, rows (pinned only), footnotes and frozen label have the shape of the `MeasurementMatrix` props. */
export interface MeasurementsModel {
  columns: { id: string; label: string; color: string }[];
  rows: { id: string; label: string; cells: Record<string, MeasurementCellModel> }[];
  footnotes: { key: number; text: string }[];
  frozenLabel?: 'Frozen' | 'Offline';
  accuracyNote: string;
  /** Every row the pin menu offers, with whether it is shown. */
  pinnable: { id: string; label: string; pinned: boolean }[];
  /** Hidden in the live window until the statistics story (D4). */
  statistics?: { title: string; headers: string[]; values: string[] };
}

export interface StatusModel {
  connection: { state: string; detail: string; tooltip: string; simulated: boolean };
  sampleRate: { value: string; tooltip: string };
  record: { points: string; span: string };
  capture: { rate: string; deadTime: string; tooltip: string };
  renderer: string;
  accuracy: string;
}

export interface GraphModel {
  state: 'live' | 'frozen' | 'empty';
  channels: { channel: ChannelNumber; samples: ArrayLike<number> }[];
  selected: ChannelNumber;
  /** Values are divisions from the top edge (0 to 8). */
  summary: string;
}

export type BannerActionId = 'copy-command' | 'retry' | 'use-simulator';

export interface BannerAction {
  id: BannerActionId;
  label: string;
}

export interface BannerModel {
  id: string;
  kind: 'info' | 'warning' | 'error';
  text: string;
  actions: BannerAction[];
}

export interface MainWindowModel {
  toolbar: ToolbarModel;
  horizontal: HorizontalModel;
  channels: ChannelModel[];
  trigger: TriggerModel;
  cursors: CursorsModel;
  measurements: MeasurementsModel;
  status: StatusModel;
  graph: GraphModel;
  banners: BannerModel[];
}
