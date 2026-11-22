import type { Snippet } from 'svelte';

/** Priorities 0 and 1 (connection, effective sample rate) never wrap. */
export interface StatusBarItem {
  id: string;
  priority: number;
  warning?: boolean;
  content: Snippet;
}
