/** `peak` is the spectrum peak marker: a down triangle that moves along the frequency axis. */
export type MarkerKind = 'ground' | 'trigger-level' | 'trigger-position' | 'peak';

/** The screen axis a ground marker belongs to: `y` in the left gutter, `x` in the bottom gutter (XY). */
export type ScreenAxis = 'x' | 'y';

export type Axis = 'vertical' | 'horizontal';

export function axisOf(kind: MarkerKind, screenAxis: ScreenAxis = 'y'): Axis {
  const horizontal = kind === 'trigger-position' || kind === 'peak' || (kind === 'ground' && screenAxis === 'x');
  return horizontal ? 'horizontal' : 'vertical';
}

const clamp01 = (value: number) => Math.min(1, Math.max(0, value));

/** Position runs 0 to 1 along the value axis: bottom to top, or left to right. */
export function positionFromPointer(axis: Axis, pointer: { x: number; y: number }, track: DOMRect): number | null {
  const length = axis === 'vertical' ? track.height : track.width;
  if (length <= 0) return null;
  if (axis === 'vertical') return clamp01(1 - (pointer.y - track.top) / length);
  return clamp01((pointer.x - track.left) / length);
}

const NUDGE_KEYS: Record<Axis, Record<string, 1 | -1>> = {
  vertical: { ArrowUp: 1, ArrowDown: -1 },
  horizontal: { ArrowRight: 1, ArrowLeft: -1 },
};

/** Alt plus arrow nudges by one step; `plainArrows` markers (peak) take the bare arrow, and Shift makes it ten steps. */
export function nudgedPosition(
  axis: Axis,
  event: KeyboardEvent,
  position: number,
  step: number,
  plainArrows = false,
): number | null {
  const direction = NUDGE_KEYS[axis][event.key];
  if (direction === undefined || !(event.altKey || plainArrows)) return null;
  const amount = plainArrows && event.shiftKey ? step * 10 : step;
  return clamp01(position + direction * amount);
}
