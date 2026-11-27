<script lang="ts">
  import type { Snippet } from 'svelte';

  const { children }: { children: Snippet } = $props();
</script>

<h3 tabindex="-1" data-step-focus>{@render children()}</h3>

<style>
  h3 {
    margin: 0;
    font-size: var(--ss-font-size-lg);
    color: var(--ss-color-text-primary);
  }
  h3:focus,
  h3:focus-visible {
    outline: none;
  }
</style>
