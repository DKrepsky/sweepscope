<script lang="ts">
  import type { ChannelModel, HorizontalModel, TriggerModel } from '../../app/main-window';
  import HorizontalGroup from './HorizontalGroup.svelte';
  import TriggerGroup from './TriggerGroup.svelte';
  import VerticalGroup from './VerticalGroup.svelte';

  interface Props {
    channels: ChannelModel[];
    horizontal: HorizontalModel;
    trigger: TriggerModel;
    /** W3: Vertical takes the first row, Horizontal and Trigger share the second. */
    wrapped: boolean;
    /** H2: dock rows use the small control height. */
    smallRows: boolean;
    /** H3: Vertical shows one channel row with tabs. */
    channelTabs: boolean;
  }

  const { channels, horizontal, trigger, wrapped, smallRows, channelTabs }: Props = $props();
</script>

<div class="dock" class:wrapped class:small={smallRows}>
  <VerticalGroup {channels} tabs={channelTabs} />
  <HorizontalGroup {horizontal} />
  <TriggerGroup {trigger} {channels} />
</div>

<style>
  .dock {
    display: grid;
    grid-template-columns: minmax(0, 1.9fr) minmax(0, 1fr) minmax(0, 1fr);
    gap: var(--ss-border-width);
    background: var(--ss-color-border);
    border-top: var(--ss-border-width) solid var(--ss-color-border);
  }
  .dock.wrapped {
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  }
  .dock.wrapped > :global(:first-child) {
    grid-column: 1 / -1;
  }
  .dock.small {
    --ss-size-control-md: var(--ss-size-control-sm);
  }
</style>
