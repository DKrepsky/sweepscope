import type { Snippet } from 'svelte';

/** Priorities 0 and 1 (connection, effective sample rate) never wrap. */
export interface StatusBarItem {
  id: string;
  priority: number;
  warning?: boolean;
  /** Sits at the right end of the first row, without a divider after it. */
  end?: boolean;
  content: Snippet;
}
