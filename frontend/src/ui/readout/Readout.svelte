<script module lang="ts">
  import type { Snippet } from 'svelte';

  export interface ReadoutProps {
    value: string;
    unit?: string;
    muted?: boolean;
    label?: Snippet;
    /** Reserved width so a changing value does not move its neighbours. */
    minWidthCh?: number;
  }
</script>

<script lang="ts">
  let { value, unit, muted = false, label, minWidthCh }: ReadoutProps = $props();
</script>

<span class={['readout', { muted }]}>
  {#if label}<span class="ss-caps">{@render label()}</span>{/if}
  <span class="value ss-num" style:min-width={minWidthCh === undefined ? undefined : `${minWidthCh}ch`}>{value}</span>
  {#if unit}<span class="unit">{unit}</span>{/if}
</span>

<style>
  .readout {
    display: inline-flex;
    align-items: baseline;
    gap: var(--ss-space-2);
    color: var(--ss-color-text-primary);
  }

  .muted {
    color: var(--ss-color-text-muted);
  }

  .value {
    display: inline-block;
    text-align: right;
    font-size: var(--ss-font-size-md);
  }

  .unit {
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-muted);
  }
</style>
