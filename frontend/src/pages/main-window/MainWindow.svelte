<script lang="ts">
  import type { Readable } from 'svelte/store';
  import type { LayoutTier } from '../../app/layout';
  import type { Notice } from '../../app/notice';
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
    notice: Readable<Notice | null>;
    onnoticedismiss: () => void;
    ontogglerun: () => void;
    ontextscale: (percent: TextScalePercent) => void;
  }

  const {
    model: modelStore,
    tier: tierStore,
    textScale: textScaleStore,
    notice: noticeStore,
    onnoticedismiss,
    ontogglerun,
    ontextscale,
  }: Props = $props();

  const MAX_BANNERS = 2;

  const model = $derived($modelStore);
  const tier = $derived($tierStore);
  const textScale = $derived($textScaleStore);
  const notice = $derived($noticeStore);

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

  const shownBanners = $derived([...model.banners].reverse().slice(0, MAX_BANNERS));
</script>

<svelte:window {onkeydown} />

<div class="viewport">
  <div class="shell" data-testid="main-window" data-width-tier={tier.width} data-height-tier={tier.height}>
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
    {#each shownBanners as banner (banner.id)}
      <Banner kind={banner.kind}>
        {#snippet text()}{banner.text}{/snippet}
      </Banner>
    {/each}
    <div class="body">
      <main class="left">
        <HorizontalReadout horizontal={model.horizontal} showOverview={tier.height === 'h1'} />
        <ScopeStage
          graph={model.graph}
          channels={model.channels}
          trigger={model.trigger}
          cursors={model.cursors}
          dots={model.horizontal.interpolation === 'dots'}
        />
        <ScaleStrip channels={model.channels} cursorSource={model.cursors.source} />
        <FrontPanelDock
          channels={model.channels}
          horizontal={model.horizontal}
          trigger={model.trigger}
          wrapped={compact}
          smallRows={tier.height !== 'h1'}
          channelTabs={tier.height === 'h3'}
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
    <MainStatusBar status={model.status} wrap={compact} />
  </div>
  {#if notice}
    <div class="toasts">
      <Toast kind={notice.kind} ondismiss={onnoticedismiss}>
        {#snippet text()}{notice.text}{/snippet}
      </Toast>
    </div>
  {/if}
</div>

<style>
  .toasts {
    position: fixed;
    inset-block-end: var(--ss-space-6);
    inset-inline-start: 50%;
    transform: translateX(-50%);
    z-index: var(--ss-z-toast);
  }
  .viewport {
    block-size: 100vh;
    overflow: auto;
  }
  .shell {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: auto minmax(0, 1fr) auto;
    min-inline-size: 64rem;
    min-block-size: 36rem;
    block-size: 100%;
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
    grid-template-rows: auto minmax(0, 1fr) auto auto;
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
