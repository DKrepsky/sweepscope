import { describe, expect, it } from 'vitest';
import { createFrameBudgetProbe, type FrameBudgetSummary } from './frame-budget-probe';

function setup(windowMs = 10_000) {
  const clock = { ms: 0 };
  const reports: FrameBudgetSummary[] = [];
  const probe = createFrameBudgetProbe({ now: () => clock.ms, report: (s) => reports.push(s), windowMs });
  const frame = (intervalMs: number, drawMs = 0) => {
    clock.ms += intervalMs;
    probe.beginFrame();
    clock.ms += drawMs;
    probe.endFrame();
    clock.ms -= drawMs;
  };
  return { clock, reports, frame };
}

describe('frame budget probe', () => {
  it('given frames inside the window, when the window has not elapsed, then nothing is reported', () => {
    const { reports, frame } = setup();
    frame(16);
    frame(16);
    expect(reports).toEqual([]);
  });

  it('given frames at 16 ms, when the window elapses, then the summary has the count, mean and max interval', () => {
    const { reports, frame } = setup(100);
    for (let i = 0; i < 8; i++) frame(16);
    expect(reports[0]).toMatchObject({ frames: 7, meanIntervalMs: 16, maxIntervalMs: 16, overBudget: 0 });
  });

  it('given one interval over 33.3 ms, when the window elapses, then it is counted over budget and is the maximum', () => {
    const { reports, frame } = setup(50);
    frame(16);
    frame(40);
    frame(16);
    expect(reports[0]).toMatchObject({ maxIntervalMs: 40, overBudget: 1 });
  });

  it('given draws of different cost, when the window elapses, then the summary has the slowest draw', () => {
    const { reports, frame } = setup(60);
    frame(20, 2);
    frame(20, 9);
    frame(20, 3);
    frame(20, 3);
    expect(reports[0]?.maxDrawMs).toBe(9);
  });

  it('given a reported window, when the next frames arrive, then the next summary starts from zero', () => {
    const { reports, frame } = setup(100);
    for (let i = 0; i < 10; i++) frame(25);
    expect(reports.map((summary) => summary.frames)).toEqual([4, 5]);
  });

  it('given the first frame only, when it is recorded, then no interval is counted', () => {
    const { reports, frame } = setup(10);
    frame(50);
    frame(50);
    expect(reports[0]?.frames).toBe(1);
  });
});
