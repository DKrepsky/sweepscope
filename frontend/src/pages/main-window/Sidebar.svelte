<script lang="ts">
  import type { CursorsModel, MeasurementsModel, PeakMode, PeakModel, ViewTab } from '../../app/main-window';
  import Badge from '../../ui/badge/Badge.svelte';
  import Panel from '../../ui/panel/Panel.svelte';
  import CursorsPanel from './CursorsPanel.svelte';
  import MeasurementsPanel from './MeasurementsPanel.svelte';
  import PeakPanel from './PeakPanel.svelte';

  interface Props {
    view: ViewTab;
    cursors: CursorsModel;
    measurements: MeasurementsModel;
    peak: PeakModel;
    onclearcursors: () => void;
    onpin: (id: string, pinned: boolean) => void;
    onpeakmode: (mode: PeakMode) => void;
  }

  const { view, cursors, measurements, peak, onclearcursors, onpin, onpeakmode }: Props = $props();
</script>

<aside class="side" data-region="sidebar" aria-label="Cursors and measurements">
  {#if view === 'fft'}
    <PeakPanel {peak} onmode={onpeakmode} />
  {:else if view === 'xy'}
    <Panel>
      {#snippet header()}Cursors{/snippet}
      {#snippet actions()}<Badge kind="software" tooltip="Cursors are not available in XY">XY</Badge>{/snippet}
      {#snippet body()}
        <p class="note">Cursors are not available in XY. Return to Yt to measure time and voltage.</p>
      {/snippet}
    </Panel>
  {:else}
    <CursorsPanel {cursors} onclear={onclearcursors} />
  {/if}
  <MeasurementsPanel {measurements} {onpin} />
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
    margin: 0;
    font-size: var(--ss-font-size-sm);
    color: var(--ss-color-text-secondary);
  }
</style>
