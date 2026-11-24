<script lang="ts">
  import type { PeakMode, PeakModel } from '../../app/main-window';
  import Panel from '../../ui/panel/Panel.svelte';
  import Segmented from '../../ui/segmented/Segmented.svelte';
  import { channelStyle } from './channel-style';

  interface Props {
    peak: PeakModel;
    onmode: (mode: PeakMode) => void;
  }

  const { peak, onmode }: Props = $props();

  let collapsed = $state(false);

  const MODES = [
    { id: 'off', label: 'Off' },
    { id: 'auto', label: 'Auto' },
    { id: 'manual', label: 'Manual' },
  ] as const;

  const options = $derived(
    MODES.map((mode) => ({ ...mode, disabled: !peak.available.enabled, disabledReason: peak.available.reason })),
  );
  const modes: readonly string[] = MODES.map(({ id }) => id);
  const choose = (id: string) => {
    if (modes.includes(id)) onmode(id as PeakMode);
  };
  const dash = (text: string) => (peak.marker ? text : '-');
</script>

<Panel {collapsed} ontoggle={() => (collapsed = !collapsed)}>
  {#snippet header()}Peak marker{/snippet}
  {#snippet actions()}
    <Segmented value={peak.mode} {options} onchange={choose} ariaLabel="Peak marker mode" />
  {/snippet}
  {#snippet body()}
    <div class="kv ss-num">
      <span class="k">Channel</span>
      <b class="v" style={channelStyle(peak.channel)}>CH{peak.channel}</b>
      <span class="k">Bin</span>
      <span class="v">{dash(String(peak.bin))}</span>
      <span class="k">Frequency</span>
      <span class="v">{peak.frequency}</span>
      <span class="k">Level</span>
      <span class="v">{peak.level}</span>
    </div>
    {#if peak.note}<p class="footnote">{peak.note}</p>{/if}
  {/snippet}
</Panel>

<style>
  .kv {
    display: grid;
    grid-template-columns: auto 1fr;
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
  b.v {
    color: var(--chc);
  }
  .footnote {
    margin-block-start: var(--ss-space-3);
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-secondary);
  }
</style>
