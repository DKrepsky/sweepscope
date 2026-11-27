<script lang="ts">
  import type { HorizontalModel, PersistenceModel, RecordingModel, RollModel, ViewsModel } from '../../app/main-window';
  import Readout from '../../ui/readout/Readout.svelte';
  import StatusChip from '../../ui/status-chip/StatusChip.svelte';

  interface Props {
    horizontal: HorizontalModel;
    views: ViewsModel;
    roll: RollModel;
    persistence: PersistenceModel;
    recording: RecordingModel;
    showOverview: boolean;
  }

  const { horizontal, views, roll, persistence, recording, showOverview }: Props = $props();

  const percent = (fraction: number) => `${fraction * 100}%`;
  const time = $derived(`${horizontal.timePerDiv.value} ${horizontal.timePerDiv.unit}`);
  const layout = $derived(views.tab === 'yt' ? (roll.active ? 'roll' : 'yt') : views.tab);
</script>

<div class="head" data-region="readout" data-layout={layout}>
  {#if layout === 'xy'}
    <Readout value="XY" />
    <span class="sep">|</span>
    <Readout value={views.xy.scales.x}>
      {#snippet label()}X{/snippet}
    </Readout>
    <span class="sep">|</span>
    <Readout value={views.xy.scales.y}>
      {#snippet label()}Y{/snippet}
    </Readout>
    <span class="sep">|</span>
    <Readout value={time}>
      {#snippet label()}Time base{/snippet}
    </Readout>
    <span class="grow"></span>
    <span class="hint">Trigger markers hidden in XY; the Trigger group stays active</span>
  {:else if layout === 'fft'}
    <Readout value={views.fft.readout.span}>
      {#snippet label()}Spectrum{/snippet}
    </Readout>
    <span class="sep">|</span>
    <Readout value={views.fft.readout.binSpacing}>
      {#snippet label()}Bin{/snippet}
    </Readout>
    <span class="sep">|</span>
    <Readout value={views.fft.windowLabel}>
      {#snippet label()}Window{/snippet}
    </Readout>
    <span class="sep">|</span>
    <Readout value={views.fft.readout.scale}>
      {#snippet label()}Scale{/snippet}
    </Readout>
    <span class="grow"></span>
    <span class="hint">Time domain overview not shown in FFT</span>
  {:else}
    <Readout value={horizontal.timePerDiv.value} unit={horizontal.timePerDiv.unit}>
      {#snippet label()}{layout === 'roll' ? 'Roll' : 'Horizontal'}{/snippet}
    </Readout>
    <span class="sep">|</span>
    {#if layout === 'yt'}
      <Readout value={horizontal.triggerPosition.value} unit={horizontal.triggerPosition.unit}>
        {#snippet label()}Trig pos{/snippet}
      </Readout>
      <span class="sep">|</span>
    {/if}
    <Readout value={layout === 'roll' ? roll.window : horizontal.window}>
      {#snippet label()}Window{/snippet}
    </Readout>
    {#if layout === 'yt' && persistence.value !== 'off'}
      <span class="sep">|</span>
      <Readout value={persistence.options.find((option) => option.value === persistence.value)?.label ?? ''}>
        {#snippet label()}Persistence{/snippet}
      </Readout>
    {/if}
    <span class="grow"></span>
    {#if layout === 'roll' && recording.chip}
      <StatusChip kind="record">{recording.chip}</StatusChip>
    {/if}
    {#if layout === 'yt' && showOverview}
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
  .hint {
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-muted);
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
