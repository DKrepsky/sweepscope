import type { StatusBarItem } from './types';

const NEVER_WRAPPED_BELOW = 2;

const byPriority = (a: StatusBarItem, b: StatusBarItem) => a.priority - b.priority;

const wraps = (item: StatusBarItem, threshold: number) =>
  item.priority >= NEVER_WRAPPED_BELOW && item.priority >= threshold;

/** First row keeps the important items; items at or above the threshold go to a second row. */
export function splitRows(items: StatusBarItem[], wrapPriorityAtLeast?: number): StatusBarItem[][] {
  const ordered = [...items].sort(byPriority);
  if (wrapPriorityAtLeast === undefined) return [ordered];
  const wrapped = ordered.filter((item) => wraps(item, wrapPriorityAtLeast));
  const first = ordered.filter((item) => !wraps(item, wrapPriorityAtLeast));
  return wrapped.length > 0 ? [first, wrapped] : [first];
}
