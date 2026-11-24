export interface PausableClock {
  now(): number;
  setPaused(paused: boolean): void;
}

/** Persistence ghosts age by this clock, so holding it still while acquisition is stopped keeps them as they are. */
export function createPausableClock(source: () => number): PausableClock {
  let skipped = 0;
  let pausedAt: number | undefined;
  return {
    now: () => (pausedAt ?? source()) - skipped,
    setPaused(paused) {
      if (paused && pausedAt === undefined) pausedAt = source();
      if (!paused && pausedAt !== undefined) {
        skipped += source() - pausedAt;
        pausedAt = undefined;
      }
    },
  };
}
