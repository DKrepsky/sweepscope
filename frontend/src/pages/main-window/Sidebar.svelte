<script lang="ts">
  import type { CursorsModel, MeasurementsModel } from '../../app/main-window';
  import CursorsPanel from './CursorsPanel.svelte';
  import MeasurementsPanel from './MeasurementsPanel.svelte';

  interface Props {
    cursors: CursorsModel;
    measurements: MeasurementsModel;
    onclearcursors: () => void;
  }

  const { cursors, measurements, onclearcursors }: Props = $props();
</script>

<aside class="side" data-region="sidebar" aria-label="Cursors and measurements">
  <CursorsPanel {cursors} onclear={onclearcursors} />
  <MeasurementsPanel {measurements} />
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
</style>
