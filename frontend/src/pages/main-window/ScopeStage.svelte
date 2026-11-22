<script lang="ts">
  import type { CursorsModel, GraphModel, TriggerModel, ChannelModel } from '../../app/main-window';
  import Badge from '../../ui/badge/Badge.svelte';
  import Graph from '../../ui/graph/Graph.svelte';
  import { screenMinimumPx } from '../../app/layout';
  import { fitScreen } from '../../ui/graph/fit-screen';
  import Marker from '../../ui/marker/Marker.svelte';
  import CursorOverlay from './CursorOverlay.svelte';

  interface Props {
    graph: GraphModel;
    channels: ChannelModel[];
    trigger: TriggerModel;
    cursors: CursorsModel;
    dots: boolean;
    /** 1 rem in px (layout store): the screen minimum is in rem. */
    rootFontPx: number;
  }

  const { graph, channels, trigger, cursors, dots, rootFontPx }: Props = $props();

  let stageSize = $state({ width: 0, height: 0 });
  let stage: HTMLElement;

  const screen = $derived.by(() => {
    const minimum = screenMinimumPx(rootFontPx);
    const fitted = fitScreen(stageSize, rootFontPx);
    return { width: Math.max(fitted.width, minimum.width), height: Math.max(fitted.height, minimum.height) };
  });
  const enabled = $derived(channels.filter((channel) => channel.enabled));

  $effect(() => {
    if (typeof ResizeObserver === 'undefined') return;
    const observer = new ResizeObserver(([entry]) => {
      if (entry) stageSize = { width: entry.contentRect.width, height: entry.contentRect.height };
    });
    observer.observe(stage);
    return () => observer.disconnect();
  });
</script>

<section class="stage" data-region="scope" aria-label="Waveform display" bind:this={stage}>
  <!-- The screen is a Tab stop for keyboard gestures (design 10.4). -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <div
    class="screen"
    style:width="{screen.width}px"
    style:height="{screen.height}px"
    tabindex="0"
    role="group"
    aria-label="Scope screen"
  >
    <Graph frame={graph} summary={graph.summary} {dots}>
      {#snippet overlay()}
        <CursorOverlay {cursors} />
        <div class="corner">
          <Badge kind="software" tooltip="Interpolation is computed in software">sin(x)/x SW</Badge>
        </div>
      {/snippet}
      {#snippet gutterLeft()}
        {#each enabled as channel (channel.number)}
          <Marker
            kind="ground"
            position={channel.groundPosition}
            label={String(channel.number)}
            name="Channel {channel.number} ground"
            color="var(--ss-color-ch{channel.number})"
          />
        {/each}
      {/snippet}
      {#snippet gutterRight()}
        <Marker kind="trigger-level" position={trigger.levelPosition} label="T" name="Trigger level">
          {#snippet tag()}{trigger.levelTag}{/snippet}
        </Marker>
      {/snippet}
      {#snippet gutterTop()}
        <Marker kind="trigger-position" position={trigger.timePosition} label="T" name="Trigger position">
          {#snippet tag()}{trigger.positionTag}{/snippet}
        </Marker>
      {/snippet}
    </Graph>
  </div>
</section>

<style>
  .stage {
    display: grid;
    place-items: center;
    min-block-size: 0;
    min-inline-size: 0;
    overflow: hidden;
    padding: 1.25rem 2rem 1rem;
    background: var(--ss-color-bg-scope);
  }
  .screen {
    position: relative;
    flex: none;
  }
  .corner {
    position: absolute;
    inset-block-end: var(--ss-space-3);
    inset-inline-start: var(--ss-space-3);
  }
</style>
