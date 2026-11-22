<script lang="ts">
  import type { ChannelModel } from '../../app/main-window';
  import Segmented from '../../ui/segmented/Segmented.svelte';
  import ChannelRow from './ChannelRow.svelte';
  import DockGroup from './DockGroup.svelte';

  interface Props {
    channels: ChannelModel[];
    /** H3: only the chosen channel's row is shown, with channel tabs. */
    tabs: boolean;
  }

  const { channels, tabs }: Props = $props();

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
  {#each visible as channel (channel.number)}
    <ChannelRow {channel} />
  {/each}
</DockGroup>
