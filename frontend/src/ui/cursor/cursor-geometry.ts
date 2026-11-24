export type CursorAxis = 'x' | 'y';

const STEPS_ACROSS: Record<CursorAxis, number> = { x: 250, y: 200 };
const SHIFT_STEPS = 10;

const clamp01 = (value: number) => Math.min(1, Math.max(0, value));

/** x runs from the left edge, y from the top edge, both 0 to 1 of the screen. */
export function positionFromPointer(
  axis: CursorAxis,
  pointer: { x: number; y: number },
  track: DOMRect,
): number | null {
  const length = axis === 'x' ? track.width : track.height;
  if (length <= 0) return null;
  return clamp01(axis === 'x' ? (pointer.x - track.left) / length : (pointer.y - track.top) / length);
}

const DIRECTIONS: Record<CursorAxis, Record<string, 1 | -1>> = {
  x: { ArrowRight: 1, ArrowLeft: -1 },
  y: { ArrowDown: 1, ArrowUp: -1 },
};

/** Arrow keys move one step towards the arrow's side of the screen; Shift moves ten. */
export function steppedPosition(axis: CursorAxis, event: KeyboardEvent, position: number): number | null {
  const direction = DIRECTIONS[axis][event.key];
  if (direction === undefined) return null;
  const steps = event.shiftKey ? SHIFT_STEPS : 1;
  return clamp01(position + (direction * steps) / STEPS_ACROSS[axis]);
}
