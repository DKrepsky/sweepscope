<script lang="ts">
  import { Badge } from '../../ui/badge';
  import { Button } from '../../ui/button';
  import { MeasurementMatrix } from '../../ui/measurement-matrix';
  import { Readout } from '../../ui/readout';
  import { StatusChip } from '../../ui/status-chip';
  import type { StatusChipKind } from '../../ui/status-chip';
  import { Tooltip } from '../../ui/tooltip';
  import { matrixColumns, matrixFootnotes, matrixRows } from './mock-data';
  import Row from './Row.svelte';
  import Section from './Section.svelte';

  const kinds: StatusChipKind[] = [
    'run',
    'stop',
    'armed',
    'triggered',
    'auto',
    'disconnected',
    'error',
    'warning',
    'info',
    'offline',
    'ok',
    'record',
  ];

  let tooltipHost: HTMLElement | undefined = $state();

  $effect(() => {
    const target = tooltipHost?.querySelector('button');
    target?.dispatchEvent(new FocusEvent('focusin', { bubbles: true }));
  });
</script>

<Section component="Badge">
  <Row>
    <Badge kind="software" tooltip="Computed in software">SW</Badge>
    <Badge kind="offline" tooltip="Viewing a saved capture">Offline</Badge>
    <Badge kind="error" tooltip="Error">Error</Badge>
    <Badge kind="experimental" tooltip="Experimental (video trigger)">Exp</Badge>
  </Row>
</Section>

<Section component="StatusChip" title="StatusChip: active and dim">
  <Row label="Active">
    {#each kinds as kind (kind)}
      <StatusChip {kind}>{kind}</StatusChip>
    {/each}
    <StatusChip kind="triggered" filled>Trig'd</StatusChip>
    <StatusChip kind="triggered">Done</StatusChip>
    <StatusChip kind="record">Rec 01:23:45 &middot; 12.4 MB</StatusChip>
  </Row>
  <Row label="Dim">
    {#each kinds as kind (kind)}
      <StatusChip {kind} dim>{kind}</StatusChip>
    {/each}
  </Row>
</Section>

<Section component="Readout">
  <Row label="Readout">
    <Readout value="1.000" unit="kHz">
      {#snippet label()}Freq{/snippet}
    </Readout>
    <Readout value="1.000" unit="kHz" muted>
      {#snippet label()}Freq (muted){/snippet}
    </Readout>
  </Row>
</Section>

<Section
  component="MeasurementMatrix"
  note="Cell states: valid, invalid, stale. CH4 is omitted from the second table (channel off)."
>
  <MeasurementMatrix columns={matrixColumns} rows={matrixRows} footnotes={matrixFootnotes}>
    {#snippet columnHeader(column)}
      {#if column.id === 'ch1'}<Badge kind="warning" tooltip="Aliasing risk (illustrative)">Alias</Badge>{/if}
    {/snippet}
  </MeasurementMatrix>
  <MeasurementMatrix
    columns={matrixColumns.slice(0, 3)}
    rows={matrixRows.slice(0, 1)}
    frozenLabel="Frozen"
    footnotes={[]}
  />
</Section>

<Section component="Tooltip" note="Forced visible by dispatching focus on mount; hover shows it after 500 ms.">
  <div class="tip-room" bind:this={tooltipHost}>
    <Tooltip text="Volts per division. Scroll or Up/Down. Range from PRD 5.1 (illustrative).">
      <Button>Focused control</Button>
    </Tooltip>
  </div>
</Section>

<style>
  .tip-room {
    padding-bottom: var(--ss-space-7);
  }
</style>
