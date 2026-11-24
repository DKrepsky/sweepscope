<script lang="ts">
  import type {
    ChannelModel,
    HorizontalModel,
    MainWindowActions,
    MathModel,
    TriggerModel,
    ViewsModel,
  } from '../../app/main-window';
  import Segmented from '../../ui/segmented/Segmented.svelte';
  import HorizontalGroup from './HorizontalGroup.svelte';
  import TriggerGroup from './TriggerGroup.svelte';
  import VerticalGroup from './VerticalGroup.svelte';

  interface Props {
    channels: ChannelModel[];
    horizontal: HorizontalModel;
    views: ViewsModel;
    trigger: TriggerModel;
    math: MathModel;
    actions: MainWindowActions;
    /** W3: Vertical takes the first row, Horizontal and Trigger share the second. */
    wrapped: boolean;
    /** H2: dock rows use the small control height. */
    smallRows: boolean;
    /** H3: Vertical shows one channel row with tabs. */
    channelTabs: boolean;
    /** Last resort (h4): one group at a time, chosen with tabs. */
    groupTabs: boolean;
  }

  const { channels, horizontal, views, trigger, math, actions, wrapped, smallRows, channelTabs, groupTabs }: Props =
    $props();

  const groupOptions = [
    { id: 'vertical', label: 'Vertical' },
    { id: 'horizontal', label: 'Horizontal' },
    { id: 'trigger', label: 'Trigger' },
  ];
  let chosenGroup = $state('vertical');
  const shows = (group: string) => !groupTabs || chosenGroup === group;
</script>

<div class="dock" class:wrapped class:small={smallRows} class:tabbed={groupTabs} data-dock-tabs={groupTabs}>
  {#if groupTabs}
    <div class="tabs">
      <Segmented
        value={chosenGroup}
        options={groupOptions}
        onchange={(id) => (chosenGroup = id)}
        ariaLabel="Dock groups"
      />
    </div>
  {/if}
  {#if shows('vertical')}<VerticalGroup {channels} {math} {actions} tabs={channelTabs} />{/if}
  {#if shows('horizontal')}<HorizontalGroup {horizontal} {views} {actions} />{/if}
  {#if shows('trigger')}<TriggerGroup {trigger} {channels} {actions} />{/if}
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
  .dock.tabbed {
    grid-template-columns: minmax(0, 1fr);
  }
  .tabs {
    padding: var(--ss-space-2) var(--ss-space-4);
    background: var(--ss-color-bg-panel);
  }
  .dock.small {
    --ss-size-control-md: var(--ss-size-control-sm);
  }
</style>
