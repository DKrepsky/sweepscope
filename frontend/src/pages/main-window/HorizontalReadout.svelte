<script lang="ts">
  import type { HorizontalModel } from '../../app/main-window';
  import Badge from '../../ui/badge/Badge.svelte';
  import Readout from '../../ui/readout/Readout.svelte';

  interface Props {
    horizontal: HorizontalModel;
    showOverview: boolean;
  }

  const { horizontal, showOverview }: Props = $props();

  const percent = (fraction: number) => `${fraction * 100}%`;
</script>

<div class="head" data-region="readout">
  <Readout value={horizontal.timePerDiv.value} unit={horizontal.timePerDiv.unit}>
    {#snippet label()}Horizontal{/snippet}
  </Readout>
  {#if horizontal.unverified}
    <Badge kind="warning" tooltip={horizontal.unverified}>Unverified</Badge>
  {/if}
  <span class="sep">|</span>
  <Readout value={horizontal.triggerPosition.value} unit={horizontal.triggerPosition.unit}>
    {#snippet label()}Trig pos{/snippet}
  </Readout>
  <span class="sep">|</span>
  <Readout value={horizontal.window}>
    {#snippet label()}Window{/snippet}
  </Readout>
  <Badge
    kind="warning"
    tooltip="Layout scaffold. Every number on this page is a placeholder, not a hardware statement."
  >
    Illustrative numbers (placeholders)
  </Badge>
  <span class="grow"></span>
  {#if showOverview}
    <span class="ss-caps">Record</span>
    <div class="overview" role="img" aria-label={horizontal.overview.summary}>
      <div
        class="window"
        style:left={percent(horizontal.overview.windowStart)}
        style:width={percent(horizontal.overview.windowSize)}
      ></div>
      <div class="tick" style:left={percent(horizontal.overview.triggerTick)}></div>
    </div>
  {/if}
</div>

<style>
  .head {
    display: flex;
    align-items: center;
    gap: var(--ss-space-5);
    min-block-size: 1.75rem;
    padding: var(--ss-space-2) var(--ss-space-4);
    background: var(--ss-color-bg-app);
    border-bottom: var(--ss-border-width) solid var(--ss-color-border-subtle);
  }
  .sep {
    color: var(--ss-color-text-muted);
  }
  .grow {
    flex: 1;
  }
  .overview {
    position: relative;
    inline-size: 15rem;
    block-size: 0.75rem;
    background: var(--ss-color-bg-inset);
    border: var(--ss-border-width) solid var(--ss-color-border);
  }
  .window {
    position: absolute;
    inset-block: 0;
    background: var(--ss-color-bg-selected);
    border: var(--ss-border-width) solid var(--ss-color-accent);
  }
  .tick {
    position: absolute;
    inset-block: calc(-1 * var(--ss-space-1));
    inline-size: var(--ss-space-1);
    background: var(--ss-color-trigger);
  }
</style>
