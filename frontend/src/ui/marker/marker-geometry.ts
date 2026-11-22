export type MarkerKind = 'ground' | 'trigger-level' | 'trigger-position';

export type Axis = 'vertical' | 'horizontal';

export const axisOf = (kind: MarkerKind): Axis => (kind === 'trigger-position' ? 'horizontal' : 'vertical');

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

export function nudgedPosition(axis: Axis, event: KeyboardEvent, position: number, step: number): number | null {
  const direction = NUDGE_KEYS[axis][event.key];
  if (!event.altKey || direction === undefined) return null;
  return clamp01(position + direction * step);
}
