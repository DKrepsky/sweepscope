/** An interval above this misses 30 fps; the story asks for none at 60 fps (spec 1.4, #8 AC1). */
export const FRAME_BUDGET_MS = 33.3;

export interface FrameBudgetSummary {
  /** Intervals measured in the window. */
  frames: number;
  meanIntervalMs: number;
  maxIntervalMs: number;
  overBudget: number;
  maxDrawMs: number;
}

export interface FrameBudgetProbe {
  /** Call first thing in the animation callback. */
  beginFrame(): void;
  /** Call after the draw; may report the finished window. */
  endFrame(): void;
}

interface Deps {
  /** Milliseconds, monotonic (`performance.now`). */
  now: () => number;
  report: (summary: FrameBudgetSummary) => void;
  windowMs?: number;
}

const DEFAULT_WINDOW_MS = 10_000;

export const consoleReport = (summary: FrameBudgetSummary): void => {
  console.info('frame budget', summary);
};

export function createFrameBudgetProbe({ now, report, windowMs = DEFAULT_WINDOW_MS }: Deps): FrameBudgetProbe {
  let lastBegin: number | undefined;
  let windowStart: number | undefined;
  let beganAt = 0;
  let intervals: number[] = [];
  let maxDraw = 0;

  const summarise = (): FrameBudgetSummary => ({
    frames: intervals.length,
    meanIntervalMs: intervals.length ? intervals.reduce((sum, ms) => sum + ms, 0) / intervals.length : 0,
    maxIntervalMs: Math.max(0, ...intervals),
    overBudget: intervals.filter((ms) => ms > FRAME_BUDGET_MS).length,
    maxDrawMs: maxDraw,
  });

  return {
    beginFrame: () => {
      beganAt = now();
      if (lastBegin !== undefined) intervals.push(beganAt - lastBegin);
      lastBegin = beganAt;
      windowStart ??= beganAt;
    },
    endFrame: () => {
      const finished = now();
      maxDraw = Math.max(maxDraw, finished - beganAt);
      if (windowStart === undefined || finished - windowStart < windowMs) return;
      report(summarise());
      intervals = [];
      maxDraw = 0;
      windowStart = undefined;
    },
  };
}
