<script lang="ts">
  import type { MeasurementsModel } from '../../app/main-window';
  import Badge from '../../ui/badge/Badge.svelte';
  import Button from '../../ui/button/Button.svelte';
  import Icon from '../../ui/icon/Icon.svelte';
  import Menu from '../../ui/menu/Menu.svelte';
  import { menuTriggerAttributes } from '../../ui/menu';
  import MeasurementMatrix from '../../ui/measurement-matrix/MeasurementMatrix.svelte';
  import Panel from '../../ui/panel/Panel.svelte';
  import Segmented from '../../ui/segmented/Segmented.svelte';

  interface Props {
    measurements: MeasurementsModel;
    onpin: (id: string, pinned: boolean) => void;
  }

  const { measurements, onpin }: Props = $props();

  const noop = () => {};
  const modes = [
    { id: 'now', label: 'Now' },
    { id: 'stats', label: 'Stats' },
  ];
  let collapsed = $state(false);
  let pinAnchor = $state<{ x: number; y: number } | null>(null);
  let pinHost: HTMLElement;

  const pinItems = $derived(measurements.pinnable.map(({ id, label, pinned }) => ({ id, label, checked: pinned })));
  const noData = 'Appears when a channel is on and a waveform is shown';

  function openPinMenu() {
    const box = pinHost.getBoundingClientRect();
    pinAnchor = { x: box.left, y: box.bottom };
  }

  function togglePin(id: string) {
    const entry = measurements.pinnable.find((other) => other.id === id);
    if (entry) onpin(id, !entry.pinned);
  }
</script>

{#snippet columnHeader(column: { id: string })}
  {@const alias = measurements.columns.find((other) => other.id === column.id)?.alias}
  {#if alias}<Badge kind="warning" tooltip={alias.message}>Alias</Badge>{/if}
{/snippet}

<Panel {collapsed} ontoggle={() => (collapsed = !collapsed)}>
  {#snippet header()}Measurements{/snippet}
  {#snippet actions()}
    {#if measurements.statistics}
      <Segmented value="now" options={modes} onchange={noop} ariaLabel="Measurement mode" />
    {/if}
    <span bind:this={pinHost}>
      <Button
        size="sm"
        title="Pin or unpin measurements"
        ariaLabel="Edit measurements"
        disabled={measurements.pinnable.length === 0}
        disabledReason={noData}
        onclick={openPinMenu}
        {...menuTriggerAttributes(pinAnchor !== null)}
      >
        {#snippet left()}<Icon name="plus" />{/snippet}
      </Button>
    </span>
  {/snippet}
  {#snippet body()}
    {#if measurements.unavailableNote}
      <p class="footnote" role="status">{measurements.unavailableNote}</p>
    {/if}
    <MeasurementMatrix
      columns={measurements.columns}
      rows={measurements.rows}
      footnotes={measurements.footnotes}
      frozenLabel={measurements.frozenLabel}
      {columnHeader}
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

{#if pinAnchor}
  <Menu items={pinItems} open={true} anchor={pinAnchor} onselect={togglePin} onclose={() => (pinAnchor = null)} />
{/if}

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
