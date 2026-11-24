<script module lang="ts">
  import type { Snippet } from 'svelte';

  export interface ScreenNoteProps {
    kind: 'info' | 'warning';
    title: Snippet;
    children?: Snippet;
    /** Buttons that fix the cause. */
    actions?: Snippet;
  }
</script>

<script lang="ts">
  import Icon from '../icon/Icon.svelte';

  let { kind, title, children, actions }: ScreenNoteProps = $props();
</script>

<div class="centre">
  <div class={['note', kind]} role="status">
    <Icon name={kind === 'warning' ? 'warning' : 'info'} />
    <div class="body">
      <p class="title">{@render title()}</p>
      {#if children}<p class="text">{@render children()}</p>{/if}
      {#if actions}<div class="actions">{@render actions()}</div>{/if}
    </div>
  </div>
</div>

<style>
  .centre {
    display: grid;
    place-items: center;
    inline-size: 100%;
    block-size: 100%;
    pointer-events: none;
  }

  .note {
    display: flex;
    align-items: flex-start;
    gap: var(--ss-space-4);
    max-inline-size: 24rem;
    padding: var(--ss-space-5);
    background: var(--ss-color-bg-overlay);
    border: var(--ss-border-width) solid currentColor;
    border-radius: var(--ss-radius-md);
    pointer-events: auto;
  }

  .warning {
    color: var(--ss-color-status-warning);
    border-style: dashed;
  }

  .info {
    color: var(--ss-color-status-info);
  }

  .body {
    display: grid;
    gap: var(--ss-space-3);
    color: var(--ss-color-text-primary);
  }

  .title {
    margin: 0;
    font-size: var(--ss-font-size-md);
    font-weight: var(--ss-font-weight-bold);
  }

  .text {
    margin: 0;
    font-size: var(--ss-font-size-sm);
    color: var(--ss-color-text-secondary);
  }

  .actions {
    display: flex;
    gap: var(--ss-space-3);
  }
</style>
