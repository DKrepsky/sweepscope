<script module lang="ts">
  import type { StepChipProps } from './StepChip.svelte';

  export interface StepChipsProps {
    /** Visible caption of the row. */
    caption: string;
    items: StepChipProps[];
  }
</script>

<script lang="ts">
  import StepChip from './StepChip.svelte';

  const { caption, items }: StepChipsProps = $props();
</script>

<div class="row">
  <span class="caption">{caption}</span>
  {#each items as item (item.label)}
    <StepChip {...item} />
  {/each}
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: var(--ss-space-3);
    padding: var(--ss-space-3) var(--ss-space-5);
    border-bottom: var(--ss-border-width) solid var(--ss-color-border-subtle);
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-secondary);
  }
  .caption {
    text-transform: uppercase;
    letter-spacing: var(--ss-letter-spacing-caps);
  }
</style>
