<script lang="ts">
  import type { MeasurementsModel } from '../../app/main-window';
  import Button from '../../ui/button/Button.svelte';
  import Icon from '../../ui/icon/Icon.svelte';
  import MeasurementMatrix from '../../ui/measurement-matrix/MeasurementMatrix.svelte';
  import Panel from '../../ui/panel/Panel.svelte';
  import Segmented from '../../ui/segmented/Segmented.svelte';

  interface Props {
    measurements: MeasurementsModel;
  }

  const { measurements }: Props = $props();

  const noop = () => {};
  const modes = [
    { id: 'now', label: 'Now' },
    { id: 'stats', label: 'Stats' },
  ];
  let collapsed = $state(false);
</script>

<Panel {collapsed} ontoggle={() => (collapsed = !collapsed)}>
  {#snippet header()}Measurements{/snippet}
  {#snippet actions()}
    {#if measurements.statistics}
      <Segmented value="now" options={modes} onchange={noop} ariaLabel="Measurement mode" />
    {/if}
    <Button size="sm" title="Pin or unpin measurements" ariaLabel="Edit measurements">
      {#snippet left()}<Icon name="plus" />{/snippet}
    </Button>
  {/snippet}
  {#snippet body()}
    <MeasurementMatrix
      columns={measurements.columns}
      rows={measurements.rows}
      footnotes={measurements.footnotes}
      frozenLabel={measurements.frozenLabel}
    />
    {#if measurements.accuracyNote}
      <p class="footnote">{measurements.accuracyNote}</p>
    {/if}
    {#if measurements.statistics}
      <div class="ss-caps stats-title">{measurements.statistics.title}</div>
      <div class="stats ss-num">
        {#each measurements.statistics.headers as header (header)}<span class="h">{header}</span>{/each}
        {#each measurements.statistics.values as value, index (index)}<span>{value}</span>{/each}
      </div>
    {/if}
  {/snippet}
</Panel>

<style>
  .footnote {
    margin-block-start: var(--ss-space-3);
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-secondary);
  }
  .stats-title {
    margin-block-start: var(--ss-space-4);
  }
  .stats {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: var(--ss-space-1) var(--ss-space-2);
    margin-block-start: var(--ss-space-2);
    font-size: var(--ss-font-size-xs);
    text-align: end;
  }
  .h {
    font-size: var(--ss-font-size-2xs);
    color: var(--ss-color-text-muted);
    text-transform: uppercase;
    letter-spacing: var(--ss-letter-spacing-caps);
  }
</style>
