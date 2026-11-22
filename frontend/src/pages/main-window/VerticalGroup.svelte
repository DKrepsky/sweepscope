<script lang="ts">
  import type { ChannelModel, MainWindowActions } from '../../app/main-window';
  import Segmented from '../../ui/segmented/Segmented.svelte';
  import ChannelRow from './ChannelRow.svelte';
  import DockGroup from './DockGroup.svelte';

  interface Props {
    channels: ChannelModel[];
    actions: MainWindowActions;
    /** H3: only the chosen channel's row is shown, with channel tabs. */
    tabs: boolean;
  }

  const { channels, actions, tabs }: Props = $props();

  let chosen = $state<string | null>(null);
  const shown = $derived(chosen ?? String(channels.find((channel) => channel.selected)?.number ?? 1));
  const visible = $derived(tabs ? channels.filter((channel) => String(channel.number) === shown) : channels);
  const tabOptions = $derived(
    channels.map((channel) => ({ id: String(channel.number), label: String(channel.number) })),
  );
</script>

<DockGroup title="Vertical" region="vertical">
  {#snippet extra()}
    {#if tabs}
      <Segmented value={shown} options={tabOptions} onchange={(id) => (chosen = id)} ariaLabel="Channel tabs" />
    {/if}
  {/snippet}
  <div class="rows">
    {#each visible as channel (channel.number)}
      <ChannelRow {channel} {actions} />
    {/each}
  </div>
</DockGroup>

<style>
  /* Seven control columns that never grow past their content, a badge column that takes the rest. */
  .rows {
    display: grid;
    grid-template-columns: repeat(7, auto) minmax(0, 1fr);
    justify-content: start;
    column-gap: var(--ss-space-3);
  }
</style>
