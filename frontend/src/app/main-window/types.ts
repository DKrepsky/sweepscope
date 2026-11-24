/** View model of the main window. Strings are already formatted (units, signs, precision); the backend will feed it. */

import type { CalibrationDialogModel, CalibrationTagModel } from '../calibration';
import type { CursorId } from '../cursors';

export type { CursorId };

export type ChannelNumber = 1 | 2 | 3 | 4;

/** Backend alias warning: badge text and the one tooltip every badge of that channel shares. */
export interface AliasNotice {
  short: string;
  message: string;
}

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

export type ChipKind =
  'run' | 'stop' | 'armed' | 'triggered' | 'auto' | 'roll' | 'record' | 'disconnected' | 'error' | 'offline';

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
  /** Arms one capture; `armed` while waiting, a second press cancels. */
  single?: { armed: boolean; availability: Availability };
  tools: { cursors: boolean; math: boolean; reference: boolean };
  /** The Math tool mirrors the row M toggle; it needs a captured record. */
  mathAvailability: Availability;
  /** The Cursors tool and the C key: they need a waveform unless cursors are already shown and can be cleared. */
  cursorsAvailability: Availability;
  files: FilesAvailability;
  /** Run and Stop. */
  availability?: Availability;
  force?: Availability;
}

export type Interpolation = 'sinc' | 'linear' | 'dots';

export interface HorizontalModel {
  timePerDiv: ScaleQuantity;
  triggerPosition: Quantity;
  window: string;
  interpolation: Interpolation;
  /** True until the user picks a mode; the backend then chooses by samples per division. */
  automaticInterpolation?: boolean;
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
  /** Ground marker, 0 (bottom) to 1 (top) of the screen. */
  groundPosition: number;
  enableReason?: string;
  availability?: Availability;
  /** A sample of the newest record sits on a converter rail. */
  clipped?: boolean;
  /** Possible aliasing at the current sample rate: badge text and the one shared tooltip. */
  alias?: AliasNotice;
  /** Axis role in XY, shown as a letter after the scale. */
  role?: 'x' | 'y';
}

export type PulseCondition = 'equal' | 'notEqual' | 'greater' | 'less';

export interface PulseModel {
  condition: PulseCondition;
  width: Quantity;
  /** Set when the last typed width was rejected; the applied width stays. */
  error?: string;
}

export interface TriggerModel {
  type: 'edge' | 'pulse' | 'video';
  pulse?: PulseModel;
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
  state: 'valid' | 'invalid' | 'stale';
  text: string;
  unit?: string;
  footnote?: number;
  tooltip?: string;
  /** Amber edge: the value is suspect (aliasing); the footnote carries the reason. */
  warning?: boolean;
}

/** Columns, rows (pinned only), footnotes and frozen label have the shape of the `MeasurementMatrix` props. */
export interface MeasurementsModel {
  columns: { id: string; label: string; color: string; alias?: AliasNotice }[];
  rows: { id: string; label: string; cells: Record<string, MeasurementCellModel> }[];
  footnotes: { key: number; text: string }[];
  frozenLabel?: 'Frozen' | 'Offline';
  /** Why no values are shown (roll mode). */
  unavailableNote?: string;
  /** Every row the pin menu offers, with whether it is shown. */
  pinnable: { id: string; label: string; pinned: boolean }[];
  /** Hidden in the live window until the statistics story (D4). */
  statistics?: { title: string; headers: string[]; values: string[] };
}

export interface StatusModel {
  /** The state word is the design's: Ready, Connecting, Disconnected or Error. */
  connection: { state: 'Ready' | 'Connecting' | 'Disconnected' | 'Error'; detail: string; tooltip: string };
  sampleRate: { value: string; tooltip: string; alias?: AliasNotice };
  record: { points: string; span: string };
  capture: { rate: string; deadTime: string; tooltip: string };
  renderer: string;
}

/** A label on the screen: `edge-right` is the roll `now` mark; `channel` colours it. */
export interface GraphTag {
  id: string;
  corner: 'top-left' | 'top-right' | 'bottom-right' | 'edge-right';
  text: string;
  channel?: ChannelNumber;
}

export interface GraphModel {
  /** Whether the shown data is current; the measurements say `Frozen` from it. */
  state: 'live' | 'frozen' | 'empty';
  /** Which screen the traces belong to. */
  view: ViewTab;
  /** Acquisition is running: persistence ageing pauses when it is not. */
  running: boolean;
  /** Only a disconnected or offline scope dims the held frame; a stopped or finished one stays at full intensity. */
  dimmed: boolean;
  /** Values are divisions from the top edge (0 to 8); spectra use the same convention. */
  channels: { channel: ChannelNumber; samples: ArrayLike<number> }[];
  /** The math trace (channel 5). */
  math?: { samples: ArrayLike<number> };
  selected: ChannelNumber;
  summary: string;
  /** Labels inside the frame, graph convention: 0 left or bottom, 1 right or top. */
  ticksX?: { at: number; text: string }[];
  ticksY?: { at: number; text: string }[];
  xy?: { x: ArrayLike<number>; y: ArrayLike<number>; xChannel: ChannelNumber; yChannel: ChannelNumber };
  /** Set only while ghosts are drawn (Yt, not roll, not Off). */
  persistence?: { decayMs: number | 'infinite'; epoch: number; revision: number };
  tags: GraphTag[];
}

export type BannerActionId = 'copy-command' | 'retry' | 'use-simulator' | 'choose-new-file' | 'dismiss';

export type PersistenceValue = 'off' | '500ms' | '1s' | '2s' | 'infinite';

export type SpectrumWindowName = 'rectangular' | 'hann' | 'hamming' | 'blackman';
export type SpectrumScaleName = 'dbv' | 'linear';
export type PeakMode = 'off' | 'auto' | 'manual';
export type MathOperationName = 'add' | 'subtract' | 'multiply' | 'invertA';

export interface AxisOption {
  channel: ChannelNumber;
  availability: Availability;
}

/** Why XY cannot draw; the screen shows it as a note, `enable` names the channel its action turns on. */
export interface ScreenProblem {
  title: string;
  text: string;
  enable?: ChannelNumber;
}

export interface XyModel {
  x: ChannelNumber;
  y: ChannelNumber;
  xOptions: AxisOption[];
  yOptions: AxisOption[];
  /** `CH1 500 mV/div` and `CH2 200 mV/div`. */
  scales: { x: string; y: string };
  /** Under the Horizontal group, such as `Fs 1.25 MS/s · straight segments`. */
  readline: string;
  problem?: ScreenProblem;
}

export interface FftModel {
  window: SpectrumWindowName;
  scale: SpectrumScaleName;
  /** Why the spectrum cannot be shown (roll mode, no channel); enabled when it can. */
  available: Availability;
  ticksX: { at: number; text: string }[];
  ticksY: { at: number; text: string }[];
  windows: { value: SpectrumWindowName; label: string }[];
  /** Menu button text (`Rect` for Rectangular) and the full name. */
  windowShort: string;
  windowLabel: string;
  /** Backend texts of the Horizontal readout; `scale` is the step, such as `10 dB/div`. */
  readout: { span: string; binSpacing: string; topLabel: string; scale: string };
  /** Under the Horizontal group, such as `Fs 50 kS/s · bin 12.21 Hz (Fs / N)`. */
  readline: string;
  problem?: ScreenProblem;
}

export interface ViewsModel {
  tab: ViewTab;
  /** Whether the tab can be chosen; XY and FFT need a captured record. */
  tabAvailability: Record<ViewTab, Availability>;
  xy: XyModel;
  fft: FftModel;
}

export interface PersistenceModel {
  value: PersistenceValue;
  options: { value: PersistenceValue; label: string }[];
  tag: string;
  available: Availability;
  /** Changes when the user clears; the graph drops its ghosts on a new value. */
  epoch: number;
}

export interface PeakModel {
  mode: PeakMode;
  channel: ChannelNumber;
  bin: number;
  /** Texts for the panel and the marker tag, one source. */
  frequency: string;
  level: string;
  available: Availability;
  /** Footnote of the panel: `Frequency resolution is one bin (12.21 Hz).` */
  note: string;
  /** Bins in the shown spectrum, 0 before the first one; a click on the screen turns into a bin of it. */
  bins: number;
  /** Absent while the marker is off or there is no spectrum; its text equals the panel's. */
  marker?: PeakMarkerModel;
}

export interface PeakMarkerModel {
  /** Along the frequency axis, 0 left to 1 right, and the level, 0 bottom to 1 top. */
  position: number;
  level: number;
  /** One bin as a fraction of the axis, so an arrow key moves one bin. */
  step: number;
  /** Shown and spoken, `1.001 kHz −6.9 dBV`. */
  valueText: string;
}

export interface GapModel {
  /** Fractions of the visible window, 0 (left) to 1 (right). */
  from: number;
  to: number;
  label: string;
}

export interface RollModel {
  active: boolean;
  /** Readout text such as `WINDOW 10 s`. */
  window: string;
  gaps: GapModel[];
  gapCount: number;
  /** Polite live-region text for the newest gap, such as `Data gap of 1.2 s`; empty without one. */
  announcement: string;
}

export interface RecordingModel {
  active: boolean;
  /** Text of the readout chip, such as `Rec 01:23:45 · 12.4 MB`. */
  chip?: string;
  available: Availability;
  failure?: { text: string; fileName: string };
}

export interface AutosetModel {
  busy: boolean;
  noSignal: boolean;
  available: Availability;
}

export interface MathRowModel {
  enabled: boolean;
  operation: MathOperationName;
  sourceA: ChannelNumber;
  sourceB?: ChannelNumber;
  scale: ScaleQuantity;
  position: Quantity;
  /** The backend label, such as `A−B`. */
  label: string;
  status: 'ok' | 'mixed' | 'unavailable';
  /** Tooltip of the status badge. */
  statusDetail?: string;
  availability: Availability;
}

export interface MathModel {
  row: MathRowModel;
  /** Scale strip chip text, empty while math is off. */
  chip: string;
  unavailableReason?: string;
}

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

/** Zero calibration: the status bar tag, and the dialog while a session exists (undefined otherwise). */
export interface CalibrationModel {
  tag: CalibrationTagModel;
  dialog: CalibrationDialogModel | undefined;
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
  views: ViewsModel;
  persistence: PersistenceModel;
  peak: PeakModel;
  roll: RollModel;
  recording: RecordingModel;
  autoset: AutosetModel;
  math: MathModel;
  calibration: CalibrationModel;
}
