const SAMPLES = 500;
const DIVISIONS = 10;

function trace(valueAt: (division: number) => number): number[] {
  return Array.from({ length: SAMPLES }, (_, index) => valueAt((index / (SAMPLES - 1)) * DIVISIONS));
}

/** Deterministic stand-in for noise: the mock must render identically every time. */
const jitter = (division: number, amount: number) => Math.sin(division * 97.3) * amount;

const phase = (division: number, period: number, offset: number) => (((division - offset) % period) + period) % period;

export function squareWave(): number[] {
  return trace((x) => (phase(x, 5, 3) < 2.5 ? 2.2 : 6) + jitter(x, 0.02));
}

export function sineWave(): number[] {
  return trace((x) => 1 - 0.8 * Math.sin((2 * Math.PI * (x - 3)) / 2.5) + jitter(x, 0.03));
}

export function triangleWave(): number[] {
  return trace((x) => {
    const t = phase(x, 5, 1) / 5;
    return 4 + (t < 0.5 ? 4 * t - 1 : 3 - 4 * t) + jitter(x, 0.04);
  });
}
