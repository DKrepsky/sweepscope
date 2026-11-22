export type ChannelNumber = 1 | 2 | 3 | 4;

export type GraphState = 'live' | 'frozen' | 'empty';

/** Samples are spread evenly over the 10 screen divisions; values are divisions from the top edge (0 to 8). */
export interface Trace {
  samples: ArrayLike<number>;
}

export interface ChannelTrace extends Trace {
  channel: ChannelNumber;
}

/** Plain data, already scaled to screen divisions, as the backend display module produces it. */
export interface GraphFrame {
  state: GraphState;
  channels: readonly ChannelTrace[];
  reference?: Trace;
  math?: Trace;
  selected?: ChannelNumber;
  dots?: boolean;
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
