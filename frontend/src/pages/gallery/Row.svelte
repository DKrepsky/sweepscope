<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    label?: string;
    children: Snippet;
  }

  const { label, children }: Props = $props();
</script>

<div class="row">
  {#if label}<span class="label">{label}</span>{/if}
  {@render children()}
</div>

<style>
  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--ss-space-4);
    margin-bottom: var(--ss-space-4);
  }

  .label {
    inline-size: 7rem;
    color: var(--ss-color-text-secondary);
    font-size: var(--ss-font-size-xs);
    letter-spacing: var(--ss-letter-spacing-caps);
    text-transform: uppercase;
  }
</style>
