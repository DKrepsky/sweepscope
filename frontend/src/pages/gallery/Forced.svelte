<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    /** A screenshot shows the state without a pointer or keyboard. */
    state: 'hover' | 'pressed' | 'focus';
    children: Snippet;
  }

  const { state, children }: Props = $props();
</script>

<span class="forced" data-force-state={state}>{@render children()}</span>

<style>
  .forced {
    display: inline-flex;
  }

  [data-force-state='hover'] :global(.btn) {
    background: var(--ss-color-bg-hover);
  }

  [data-force-state='pressed'] :global(.btn) {
    background: var(--ss-color-bg-pressed);
  }

  [data-force-state='focus'] :global(.btn) {
    outline: var(--ss-border-width-focus) solid var(--ss-color-focus);
    outline-offset: var(--ss-border-width);
  }
</style>
