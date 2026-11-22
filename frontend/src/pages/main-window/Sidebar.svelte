<script lang="ts">
  import type { CursorsModel, MeasurementsModel } from '../../app/main-window';
  import Button from '../../ui/button/Button.svelte';
  import Icon from '../../ui/icon/Icon.svelte';
  import MeasurementMatrix from '../../ui/measurement-matrix/MeasurementMatrix.svelte';
  import Panel from '../../ui/panel/Panel.svelte';
  import Segmented from '../../ui/segmented/Segmented.svelte';
  import { channelStyle } from './channel-style';

  interface Props {
    cursors: CursorsModel;
    measurements: MeasurementsModel;
  }

  const { cursors, measurements }: Props = $props();

  const noop = () => {};
  const modes = [
    { id: 'now', label: 'Now' },
    { id: 'stats', label: 'Stats' },
  ];
  let cursorsCollapsed = $state(false);
  let measurementsCollapsed = $state(false);
</script>

<aside class="side" data-region="sidebar" aria-label="Cursors and measurements">
  <Panel collapsed={cursorsCollapsed} ontoggle={() => (cursorsCollapsed = !cursorsCollapsed)}>
    {#snippet header()}Cursors{/snippet}
    {#snippet actions()}
      <span class="note">on <b class="ss-num" style={channelStyle(cursors.source)}>CH{cursors.source}</b></span>
      <Button size="sm" title="Remove all cursors (double-click a cursor removes it)">Clear</Button>
    {/snippet}
    {#snippet body()}
      <div class="kv ss-num">
        {#each cursors.values as entry (entry.label)}
          <span class="k">{entry.label}</span>
          <span class="v" class:strong={entry.emphasised}>{entry.value}</span>
        {/each}
      </div>
      <p class="footnote">{cursors.originNote}</p>
    {/snippet}
  </Panel>

  <Panel collapsed={measurementsCollapsed} ontoggle={() => (measurementsCollapsed = !measurementsCollapsed)}>
    {#snippet header()}Measurements{/snippet}
    {#snippet actions()}
      <Segmented value="now" options={modes} onchange={noop} ariaLabel="Measurement mode" />
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
      <div class="ss-caps stats-title">{measurements.statistics.title}</div>
      <div class="stats ss-num">
        {#each measurements.statistics.headers as header (header)}<span class="h">{header}</span>{/each}
        {#each measurements.statistics.values as value, index (index)}<span>{value}</span>{/each}
      </div>
    {/snippet}
  </Panel>
</aside>

<style>
  .side {
    display: flex;
    flex-direction: column;
    min-block-size: 0;
    overflow: auto;
    background: var(--ss-color-bg-panel);
    border-inline-start: var(--ss-border-width) solid var(--ss-color-border);
  }
  .note {
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-secondary);
  }
  .note b {
    color: var(--chc);
  }
  .kv {
    display: grid;
    grid-template-columns: auto 1fr auto 1fr;
    gap: var(--ss-space-2) var(--ss-space-4);
    align-items: baseline;
    font-size: var(--ss-font-size-md);
  }
  .k {
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-secondary);
  }
  .v {
    text-align: end;
  }
  .strong {
    color: var(--ss-color-text-primary);
  }
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
