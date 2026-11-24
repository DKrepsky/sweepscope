export type ChannelNumber = 1 | 2 | 3 | 4;

export type GraphState = 'live' | 'frozen' | 'empty';

/** Samples are spread evenly over the 10 screen divisions; values are divisions from the top edge (0 to 8). */
export interface Trace {
  samples: ArrayLike<number>;
}

export interface ChannelTrace extends Trace {
  channel: ChannelNumber;
}

/** A label inside the frame; `at` runs 0 to 1 from the left (X) or from the bottom (Y), like a Marker position. */
export interface Tick {
  at: number;
  text: string;
}

/** XY pair: X and Y use the Yt convention, divisions from the top edge with 4 at the centre. */
export interface XyTrace {
  x: ArrayLike<number>;
  y: ArrayLike<number>;
  xChannel: ChannelNumber;
  yChannel: ChannelNumber;
}

/**
 * Ghosts of past frames in Yt. `epoch` and `revision` clear them when they change; `now` is the only clock, so a
 * caller that holds it still (stopped) pauses the ageing.
 */
export interface PersistenceOptions {
  decayMs: number | 'infinite';
  epoch: number;
  revision?: number;
  now: () => number;
}

/** Plain data, already scaled to screen divisions, as the backend display module produces it. */
export interface GraphFrame {
  state: GraphState;
  channels: readonly ChannelTrace[];
  reference?: Trace;
  math?: Trace;
  selected?: ChannelNumber;
  dots?: boolean;
  ticksX?: readonly Tick[];
  ticksY?: readonly Tick[];
  xy?: XyTrace;
  persistence?: PersistenceOptions;
}

/** CSS pixel size of the screen and the device pixel ratio it is drawn at. */
export interface PixelSize {
  width: number;
  height: number;
  ratio: number;
}

export interface TraceRenderer {
  draw(frame: GraphFrame, size: PixelSize): void;
  dispose(): void;
}

export const EMPTY_FRAME: GraphFrame = { state: 'empty', channels: [] };
