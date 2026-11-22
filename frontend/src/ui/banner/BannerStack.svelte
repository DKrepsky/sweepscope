<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    /** Chronological: the last entry is the newest. */
    banners: { id: string; node: Snippet }[];
    max?: number;
  }

  const { banners, max = 2 }: Props = $props();

  const visible = $derived([...banners].reverse().slice(0, max));
</script>

<div class="stack">
  {#each visible as banner (banner.id)}
    {@render banner.node()}
  {/each}
</div>

<style>
  .stack {
    display: grid;
    gap: var(--ss-space-2);
  }
</style>
