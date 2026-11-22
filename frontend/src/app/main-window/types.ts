/** View model of the main window. Strings are already formatted (units, signs, precision); the backend will feed it. */

export type ChannelNumber = 1 | 2 | 3 | 4;

export interface Quantity {
  value: string;
  unit: string;
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

export interface ToolbarModel {
  running: boolean;
  chips: ChipModel[];
  persistence: string;
  view: ViewTab;
  tools: { cursors: boolean; math: boolean; reference: boolean };
}

export type Interpolation = 'sinc' | 'linear' | 'dots';

export interface HorizontalModel {
  timePerDiv: Quantity;
  triggerPosition: Quantity;
  window: string;
  interpolation: Interpolation;
  /** Effective sample rate line of the Horizontal group. */
  rateNote: string;
  overview: { windowStart: number; windowSize: number; triggerTick: number; summary: string };
}

export interface ChannelModel {
  number: ChannelNumber;
  enabled: boolean;
  selected: boolean;
  voltsPerDiv: Quantity;
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
}

export interface CursorValue {
  label: string;
  value: string;
  emphasised?: boolean;
}

export interface CursorsModel {
  enabled: boolean;
  source: ChannelNumber;
  values: CursorValue[];
  originNote: string;
  /** Screen overlay, fractions of the screen: x from the left, y from the top. */
  lines: { x: number[]; y: number[] };
  tags: { text: string; x?: number; y?: number; anchor: 'bottom' | 'top' | 'left' }[];
}

export interface MeasurementCellModel {
  state: 'valid' | 'unverified' | 'invalid' | 'stale';
  text: string;
  unit?: string;
  footnote?: number;
  tooltip?: string;
}

/** Same shape as the props of the `MeasurementMatrix` component. */
export interface MeasurementsModel {
  columns: { id: string; label: string; color: string }[];
  rows: { id: string; label: string; cells: Record<string, MeasurementCellModel> }[];
  footnotes: { key: number; text: string }[];
  frozenLabel?: 'Frozen' | 'Offline';
  statistics: { title: string; headers: string[]; values: string[] };
}

export interface StatusModel {
  connection: { state: string; detail: string; tooltip: string };
  sampleRate: { value: string; tooltip: string };
  record: { points: string; span: string };
  capture: { rate: string; deadTime: string; tooltip: string };
  renderer: string;
  accuracy: string;
}

export interface GraphModel {
  state: 'live' | 'frozen' | 'empty';
  channels: { channel: ChannelNumber; samples: number[] }[];
  selected: ChannelNumber;
  /** Values are divisions from the top edge (0 to 8). */
  summary: string;
}

export interface BannerModel {
  id: string;
  kind: 'info' | 'warning' | 'error';
  text: string;
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
