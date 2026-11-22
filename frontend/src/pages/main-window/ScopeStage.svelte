<script lang="ts">
  import type { CursorsModel, GraphModel, TriggerModel, ChannelModel } from '../../app/main-window';
  import Badge from '../../ui/badge/Badge.svelte';
  import Graph from '../../ui/graph/Graph.svelte';
  import { fitScreen } from '../../ui/graph/fit-screen';
  import Marker from '../../ui/marker/Marker.svelte';
  import CursorOverlay from './CursorOverlay.svelte';

  interface Props {
    graph: GraphModel;
    channels: ChannelModel[];
    trigger: TriggerModel;
    cursors: CursorsModel;
    dots: boolean;
  }

  const { graph, channels, trigger, cursors, dots }: Props = $props();

  let stageSize = $state({ width: 0, height: 0 });
  let stage: HTMLElement;

  const screen = $derived(fitScreen(stageSize));
  const enabled = $derived(channels.filter((channel) => channel.enabled));

  $effect(() => {
    const measure = (width: number, height: number) => (stageSize = { width, height });
    if (typeof ResizeObserver === 'undefined') return;
    const observer = new ResizeObserver(([entry]) => {
      if (entry) measure(entry.contentRect.width, entry.contentRect.height);
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
