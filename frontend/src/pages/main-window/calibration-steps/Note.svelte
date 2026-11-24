<script module lang="ts">
  import type { Snippet } from 'svelte';

  export interface NoteProps {
    kind: 'info' | 'good' | 'error';
    role?: 'status' | 'alert';
    /** The note takes the focus when a step opens and nothing else in the step can. */
    focusTarget?: boolean;
    children: Snippet;
  }
</script>

<script lang="ts">
  import Icon from '../../../ui/icon/Icon.svelte';

  const { kind, role, focusTarget = false, children }: NoteProps = $props();
  const icon = $derived(kind === 'good' ? 'ok' : kind === 'error' ? 'warning' : 'info');
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div class="note {kind}" {role} data-step-focus={focusTarget ? '' : undefined} tabindex={focusTarget ? -1 : undefined}>
  <Icon name={icon} />
  <span>{@render children()}</span>
</div>

<style>
  .note {
    display: flex;
    gap: var(--ss-space-3);
    align-items: flex-start;
    padding: var(--ss-space-3) var(--ss-space-4);
    background: var(--ss-color-bg-inset);
    border-inline-start: var(--ss-space-1) solid var(--ss-color-status-info);
    font-size: var(--ss-font-size-xs);
  }
  .note[tabindex='-1']:focus,
  .note[tabindex='-1']:focus-visible {
    outline: none;
  }
  .good {
    border-inline-start-color: var(--ss-color-status-ok);
  }
  .error {
    border-inline-start-color: var(--ss-color-status-error);
  }
  .note :global(svg) {
    flex: none;
    inline-size: 0.75rem;
    block-size: 0.75rem;
  }
  .note :global(b) {
    font-weight: var(--ss-font-weight-bold);
    color: var(--ss-color-text-primary);
  }
</style>
