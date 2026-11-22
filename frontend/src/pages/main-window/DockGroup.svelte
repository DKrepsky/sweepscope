<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title: string;
    region: string;
    extra?: Snippet;
    children: Snippet;
  }

  const { title, region, extra, children }: Props = $props();
</script>

<section class="group" data-region={region} aria-label={title}>
  <header>
    <h2>{title}</h2>
    {@render extra?.()}
    <span class="rule"></span>
  </header>
  {@render children()}
</section>

<style>
  .group {
    display: flex;
    flex-direction: column;
    gap: var(--ss-space-3);
    min-inline-size: 0;
    padding: var(--ss-space-3) var(--ss-space-4) var(--ss-space-4);
    background: var(--ss-color-bg-panel);
  }
  header {
    display: flex;
    align-items: center;
    gap: var(--ss-space-3);
  }
  h2 {
    font-size: var(--ss-font-size-2xs);
    letter-spacing: var(--ss-letter-spacing-caps);
    text-transform: uppercase;
    color: var(--ss-color-text-secondary);
    font-weight: var(--ss-font-weight-bold);
  }
  .rule {
    flex: 1;
    block-size: var(--ss-border-width);
    background: var(--ss-color-border-subtle);
  }
</style>
