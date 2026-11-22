<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    /** Component name as listed in design 9; tests and screenshots find the section by it. */
    component: string;
    title?: string;
    note?: string;
    children: Snippet;
  }

  const { component, title = component, note, children }: Props = $props();
</script>

<section class="section" data-component={component}>
  <h2>{title}</h2>
  {#if note}<p class="note">{note}</p>{/if}
  {@render children()}
</section>

<style>
  .section {
    border: var(--ss-border-width) solid var(--ss-color-border);
    background: var(--ss-color-bg-panel);
    padding: var(--ss-space-4) var(--ss-space-5) var(--ss-space-5);
  }

  h2 {
    margin-bottom: var(--ss-space-4);
    padding-bottom: var(--ss-space-2);
    border-bottom: var(--ss-border-width) solid var(--ss-color-border-subtle);
    color: var(--ss-color-text-secondary);
    font-size: var(--ss-font-size-sm);
    letter-spacing: var(--ss-letter-spacing-caps);
    text-transform: uppercase;
  }

  .note {
    margin-bottom: var(--ss-space-4);
    color: var(--ss-color-text-muted);
    font-size: var(--ss-font-size-xs);
  }
</style>
