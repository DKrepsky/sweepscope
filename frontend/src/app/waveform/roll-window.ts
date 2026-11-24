import type { RollItem } from './decode-roll';
import type { TraceChannel } from './trace-channel';

export const ROLL_WINDOW_SAMPLES = 2500;

/** A stretch of missed samples as fractions of the window width (0 left, 1 right). */
export interface RollGap {
  from: number;
  to: number;
  missed: number;
  /** The samples are unknown, so where the stretch ends is the backend's estimate. */
  approximate: true;
}

export interface RollView {
  /** Divisions from the top edge, newest at the right; NaN where there is no sample. */
  channels: { channel: TraceChannel; samples: Float32Array }[];
  gaps: RollGap[];
  /** Gaps seen in this session, including those that have scrolled out. */
  gapCount: number;
}

export interface RollWindow {
  push(item: RollItem): void;
  view(): RollView;
}

interface MissedRun {
  /** Index of the last sample before the run. */
  after: number;
  missed: number;
}

/** A fixed-size window of the newest samples: memory never grows however long the roll runs. */
class Window implements RollWindow {
  private readonly traces = new Map<TraceChannel, Float32Array>();
  private session: number | undefined;
  private nextIndex: number | undefined;
  private runs: MissedRun[] = [];
  private gapCount = 0;

  constructor(private readonly size: number) {}

  push(item: RollItem): void {
    if (item.session !== this.session) this.restart(item.session);
    if (item.kind === 'samples') this.pushSamples(item);
    if (item.kind === 'gap') this.skipTo(item.firstIndex + 1 + item.missed, item.firstIndex + 1);
    this.forgetScrolledOut();
  }

  view(): RollView {
    const start = this.startIndex();
    const fraction = (index: number) => Math.min(1, Math.max(0, (index - start) / this.size));
    return {
      channels: [...this.traces]
        .sort(([a], [b]) => a - b)
        .map(([channel, samples]) => ({ channel, samples: samples.slice() })),
      gaps: this.runs.map(({ after, missed }) => ({
        from: fraction(after + 1),
        to: fraction(after + 1 + missed),
        missed,
        approximate: true,
      })),
      gapCount: this.gapCount,
    };
  }

  private startIndex(): number {
    return (this.nextIndex ?? 0) - this.size;
  }

  private restart(session: number): void {
    this.session = session;
    this.nextIndex = undefined;
    this.traces.clear();
    this.runs = [];
    this.gapCount = 0;
  }

  private scrollBy(count: number): void {
    const shift = Math.min(count, this.size);
    this.traces.forEach((trace) => {
      trace.copyWithin(0, shift);
      trace.fill(NaN, this.size - shift);
    });
    this.nextIndex = (this.nextIndex ?? 0) + count;
  }

  /** Samples from `nextIndex` up to `index` never arrived; `start` places the first item of a session. */
  private skipTo(index: number, start: number): void {
    this.nextIndex ??= start;
    if (index <= this.nextIndex) return;
    this.runs.push({ after: this.nextIndex - 1, missed: index - this.nextIndex });
    this.gapCount++;
    this.scrollBy(index - this.nextIndex);
  }

  private pushSamples(item: RollItem): void {
    this.nextIndex ??= item.firstIndex;
    if (item.firstIndex < this.nextIndex) return;
    this.skipTo(item.firstIndex, item.firstIndex);
    item.channels.forEach(({ channel }) => {
      if (!this.traces.has(channel)) this.traces.set(channel, new Float32Array(this.size).fill(NaN));
    });
    this.scrollBy(item.count);
    const kept = Math.min(item.count, this.size);
    item.channels.forEach(({ channel, samples }) => {
      this.traces.get(channel)?.set(samples.subarray(item.count - kept), this.size - kept);
    });
  }

  private forgetScrolledOut(): void {
    const start = this.startIndex();
    this.runs = this.runs.filter((run) => run.after + 1 + run.missed > start);
  }
}

export function createRollWindow(size = ROLL_WINDOW_SAMPLES): RollWindow {
  return new Window(size);
}
