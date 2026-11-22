<script lang="ts">
  import type { Readable } from 'svelte/store';
  import { heightCollapse, screenMinimumPx, type LayoutTier } from '../../app/layout';
  import type { Notification } from '../../app/notifications';
  import type { MainWindowModel } from '../../app/main-window';
  import type { TextScalePercent } from '../../app/text-scale';
  import Banner from '../../ui/banner/Banner.svelte';
  import Toast from '../../ui/toast/Toast.svelte';
  import FrontPanelDock from './FrontPanelDock.svelte';
  import HorizontalReadout from './HorizontalReadout.svelte';
  import MainStatusBar from './MainStatusBar.svelte';
  import ScaleStrip from './ScaleStrip.svelte';
  import ScopeStage from './ScopeStage.svelte';
  import Sidebar from './Sidebar.svelte';
  import Toolbar from './Toolbar.svelte';

  interface Props {
    model: Readable<MainWindowModel>;
    tier: Readable<LayoutTier>;
    textScale: Readable<TextScalePercent>;
    notifications: Readable<Notification[]>;
    onnotificationdismiss: (id: number) => void;
    ontogglerun: () => void;
    ontextscale: (percent: TextScalePercent) => void;
  }

  const {
    model: modelStore,
    tier: tierStore,
    textScale: textScaleStore,
    notifications: notificationsStore,
    onnotificationdismiss,
    ontogglerun,
    ontextscale,
  }: Props = $props();

  const MAX_BANNERS = 2;

  const model = $derived($modelStore);
  const tier = $derived($tierStore);
  const textScale = $derived($textScaleStore);
  const notifications = $derived($notificationsStore);
  const collapse = $derived(heightCollapse(tier.height));
  // Gutters above and below the screen are 1.25 + 1 rem (design 3.3).
  const stageMinimumPx = $derived(screenMinimumPx(tier.rootFontPx).height + 2.25 * tier.rootFontPx);

  const compact = $derived(tier.width === 'w3' || tier.width === 'scroll');
  let drawerRequested = $state(false);
  const drawerOpen = $derived(compact && drawerRequested);
  let drawer = $state<HTMLElement>();

  $effect(() => {
    if (drawerOpen) drawer?.focus();
  });

  function onkeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && drawerOpen) drawerRequested = false;
  }

  const triggerSummary = $derived(
    `Trigger CH${model.trigger.source} ${model.trigger.type} ${model.trigger.slope} ${model.trigger.level.value} ${model.trigger.level.unit}`,
  );
  const shownBanners = $derived([...model.banners].reverse().slice(0, MAX_BANNERS));
</script>

<svelte:window {onkeydown} />

<div class="viewport">
  <div
    class="shell"
    data-testid="main-window"
    data-width-tier={tier.width}
    data-height-tier={tier.height}
    style:--stage-min-block="{stageMinimumPx}px"
  >
    <Toolbar
      toolbar={model.toolbar}
      toolLabels={tier.width === 'w1'}
      runLabels={!compact}
      drawerButton={compact}
      {drawerOpen}
      {textScale}
      {ontogglerun}
      ontoggledrawer={() => (drawerRequested = !drawerRequested)}
      {ontextscale}
    />
    <div class="content" data-region="content">
      {#each shownBanners as banner (banner.id)}
        <Banner kind={banner.kind}>
          {#snippet text()}{banner.text}{/snippet}
        </Banner>
      {/each}
      <div class="floor">
        <div class="body">
          <main class="left">
            <HorizontalReadout horizontal={model.horizontal} showOverview={collapse.recordOverview} />
            <ScopeStage
              rootFontPx={tier.rootFontPx}
              graph={model.graph}
              channels={model.channels}
              trigger={model.trigger}
              cursors={model.cursors}
              dots={model.horizontal.interpolation === 'dots'}
            />
            <ScaleStrip
              channels={model.channels}
              cursorSource={model.cursors.source}
              triggerSummary={collapse.groupTabs ? triggerSummary : undefined}
            />
            <FrontPanelDock
              channels={model.channels}
              horizontal={model.horizontal}
              trigger={model.trigger}
              wrapped={compact}
              smallRows={collapse.smallRows}
              channelTabs={collapse.channelTabs}
              groupTabs={collapse.groupTabs}
            />
          </main>
          {#if !compact}
            <div class="slot">
              <Sidebar cursors={model.cursors} measurements={model.measurements} />
            </div>
          {:else if drawerOpen}
            <div
              class="slot drawer"
              tabindex="-1"
              bind:this={drawer}
              aria-label="Cursors and measurements drawer"
              role="region"
            >
              <Sidebar cursors={model.cursors} measurements={model.measurements} />
            </div>
          {/if}
        </div>
      </div>
    </div>
    <MainStatusBar status={model.status} wrap={compact} essentialOnly={collapse.essentialStatusOnly} />
  </div>
  {#if notifications.length > 0}
    <div class="toasts">
      {#each notifications as notification (notification.id)}
        <Toast kind={notification.kind} ondismiss={() => onnotificationdismiss(notification.id)}>
          {#snippet text()}{notification.text}{/snippet}
        </Toast>
      {/each}
    </div>
  {/if}
</div>

<style>
  .toasts {
    position: fixed;
    display: flex;
    flex-direction: column;
    gap: var(--ss-space-3);
    inset-block-end: var(--ss-space-6);
    inset-inline-start: 50%;
    transform: translateX(-50%);
    z-index: var(--ss-z-toast);
  }
  /* Horizontal scroll moves the whole window content; vertical scroll is inside `.content` (design 3.2). */
  .viewport {
    block-size: 100vh;
    overflow-x: auto;
    overflow-y: hidden;
  }
  .shell {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: auto minmax(0, 1fr) auto;
    min-inline-size: 64rem;
    block-size: 100%;
  }
  .content {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: auto minmax(0, 1fr);
    overflow-x: hidden;
    overflow-y: auto;
    min-block-size: 0;
  }
  /* The floor is 45 rem for the window: what is left after the pinned toolbar and a one-line status bar. */
  .floor {
    grid-row: 2;
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: minmax(0, 1fr);
    min-block-size: calc(45rem - var(--ss-size-toolbar) - var(--ss-size-statusbar));
  }
  .body {
    position: relative;
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    min-block-size: 0;
  }
  .shell[data-width-tier='w3'] .body,
  .shell[data-width-tier='scroll'] .body {
    grid-template-columns: minmax(0, 1fr);
  }
  .left {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: auto minmax(var(--stage-min-block), 1fr) auto auto;
    min-inline-size: 0;
    min-block-size: 0;
  }
  .slot {
    display: grid;
    min-block-size: 0;
    inline-size: var(--ss-size-sidebar);
  }
  .drawer {
    position: absolute;
    inset-block: 0;
    inset-inline-end: 0;
    z-index: var(--ss-z-overlay);
    box-shadow: var(--ss-shadow-popover);
  }
</style>
