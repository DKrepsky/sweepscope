<script lang="ts">
  import {
    divisionsFromFraction,
    divisionsFromXFraction,
    xFractionFromGround,
    type ChannelModel,
    type CursorsModel,
    type GraphModel,
    type Interpolation,
    type MainWindowActions,
    type PeakModel,
    type RollModel,
    type TriggerModel,
    type ViewsModel,
  } from '../../app/main-window';
  import { createPausableClock } from '../../app/views';
  import Badge from '../../ui/badge/Badge.svelte';
  import Button from '../../ui/button/Button.svelte';
  import Graph from '../../ui/graph/Graph.svelte';
  import type { GraphFrame } from '../../ui/graph/graph-frame';
  import { screenMinimumPx } from '../../app/layout';
  import { fitScreen } from '../../ui/graph/fit-screen';
  import Marker from '../../ui/marker/Marker.svelte';
  import ScreenNote from '../../ui/screen-note/ScreenNote.svelte';
  import { channelStyle } from './channel-style';
  import CursorOverlay from './CursorOverlay.svelte';
  import RollGaps from './RollGaps.svelte';

  interface Props {
    graph: GraphModel;
    channels: ChannelModel[];
    trigger: TriggerModel;
    cursors: CursorsModel;
    views: ViewsModel;
    peak: PeakModel;
    roll: RollModel;
    dots: boolean;
    /** Wins over `dots` when given: the badge names the mode. */
    interpolation?: Interpolation;
    actions: MainWindowActions;
    /** 1 rem in px (layout store): the screen minimum is in rem. */
    rootFontPx: number;
    /** Milliseconds; persistence ghosts age by it, only while acquisition runs. */
    clock?: () => number;
  }

  const {
    graph,
    channels,
    trigger,
    cursors,
    views,
    peak,
    roll,
    dots,
    interpolation,
    actions: windowActions,
    rootFontPx,
    clock = () => performance.now(),
  }: Props = $props();

  const mode = $derived(interpolation ?? (dots ? 'dots' : 'linear'));
  const BADGES = { sinc: 'sin(x)/x SW', linear: 'Linear SW', dots: 'Dots' } as const;

  const triggerOff = $derived(trigger.availability?.enabled === false);
  const triggerReason = $derived(trigger.availability?.reason);

  const view = $derived(graph.view);
  const problem = $derived(view === 'xy' ? views.xy.problem : view === 'fft' ? views.fft.problem : undefined);

  let stageSize = $state({ width: 0, height: 0 });
  let stage: HTMLElement;

  const screen = $derived.by(() => {
    const minimum = screenMinimumPx(rootFontPx);
    const fitted = fitScreen(stageSize, rootFontPx);
    return { width: Math.max(fitted.width, minimum.width), height: Math.max(fitted.height, minimum.height) };
  });
  const enabled = $derived(channels.filter((channel) => channel.enabled));
  const groundMarkers = $derived(
    view === 'yt' ? enabled : view === 'xy' ? enabled.filter((channel) => channel.number === views.xy.y) : [],
  );
  const xGround = $derived(view === 'xy' ? channels.find((channel) => channel.number === views.xy.x) : undefined);

  const persistenceClock = $derived(createPausableClock(clock));
  $effect(() => persistenceClock.setPaused(!graph.running));

  /** Only a disconnected or offline scope dims the held frame (Q-M3-15). */
  const frame = $derived<GraphFrame>({
    state: graph.state === 'empty' ? 'empty' : graph.dimmed ? 'frozen' : 'live',
    channels: graph.channels,
    selected: graph.selected,
    ...(graph.math && { math: graph.math }),
    ...(graph.ticksX && { ticksX: graph.ticksX }),
    ...(graph.ticksY && { ticksY: graph.ticksY }),
    ...(graph.xy && { xy: graph.xy }),
    ...(graph.persistence && { persistence: { ...graph.persistence, now: persistenceClock.now } }),
  });

  function placePeakAt(event: MouseEvent) {
    if (event.target !== event.currentTarget || peak.bins < 2 || !peak.available.enabled) return;
    const box = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const fraction = Math.min(1, Math.max(0, (event.clientX - box.left) / box.width));
    windowActions.placePeakMarker(Math.round(fraction * (peak.bins - 1)));
  }

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
    <Graph {frame} summary={graph.summary} dots={view === 'yt' && mode === 'dots'}>
      {#snippet overlay()}
        {#if view === 'yt'}
          <CursorOverlay {cursors} onmove={windowActions.moveCursor} onremove={windowActions.removeCursor} />
        {/if}
        {#if view === 'fft' && !problem}
          <!-- Pointer placement only; the marker itself is the keyboard control. -->
          <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
          <div class="peak-layer" onclick={placePeakAt}>
            {#if peak.marker}
              {@const marker = peak.marker}
              <Marker
                kind="peak"
                position={marker.position}
                level={marker.level}
                step={marker.step}
                valueText={marker.valueText}
                label=""
                name="Peak marker"
                color="var(--ss-color-cursor)"
                oncommit={(fraction) => windowActions.placePeakMarker(Math.round(fraction / marker.step))}
              >
                {#snippet tag()}{marker.valueText}{/snippet}
              </Marker>
            {/if}
          </div>
        {/if}
        {#if roll.active && view === 'yt'}
          <RollGaps gaps={roll.gaps} announcement={roll.announcement} />
        {/if}
        {#each graph.tags as tag (tag.id)}
          <span
            class="tag ss-num {tag.corner}"
            style={tag.channel ? `${channelStyle(tag.channel)}; color: var(--chc)` : undefined}
            data-tag={tag.id}
          >
            {tag.text}
          </span>
        {/each}
        {#if problem}
          <ScreenNote kind="warning">
            {#snippet title()}{problem.title}{/snippet}
            {problem.text}
            {#snippet actions()}
              {#if problem.enable}
                {@const channel = problem.enable}
                <Button size="sm" onclick={() => windowActions.setChannelEnabled(channel, true)}>
                  Enable CH{channel}
                </Button>
              {/if}
            {/snippet}
          </ScreenNote>
        {/if}
        <div class="corner">
          {#if view === 'yt'}
            <Badge kind="software" tooltip="Interpolation is computed in software">{BADGES[mode]}</Badge>
          {/if}
        </div>
      {/snippet}
      {#snippet gutterLeft()}
        {#each groundMarkers as channel (channel.number)}
          <Marker
            kind="ground"
            position={channel.groundPosition}
            label={String(channel.number)}
            name="Channel {channel.number} ground"
            color="var(--ss-color-ch{channel.number})"
            disabled={channel.availability?.enabled === false}
            disabledReason={channel.availability?.reason}
            oncommit={(fraction) => windowActions.setChannelPosition(channel.number, divisionsFromFraction(fraction))}
            onreset={() => windowActions.setChannelPosition(channel.number, 0)}
          />
        {/each}
      {/snippet}
      {#snippet gutterRight()}
        {#if view === 'yt'}
          <Marker
            kind="trigger-level"
            position={trigger.levelPosition}
            label="T"
            name="Trigger level"
            disabled={triggerOff}
            disabledReason={triggerReason}
            oncommit={(fraction) => windowActions.setTriggerLevel(divisionsFromFraction(fraction))}
            onreset={windowActions.resetTriggerLevel}
          >
            {#snippet tag()}{trigger.levelTag}{/snippet}
          </Marker>
        {/if}
      {/snippet}
      {#snippet gutterTop()}
        {#if view === 'yt'}
          <Marker
            kind="trigger-position"
            position={trigger.timePosition}
            label="T"
            name="Trigger position"
            disabled={triggerOff}
            disabledReason={triggerReason}
            oncommit={windowActions.setTriggerPosition}
          >
            {#snippet tag()}{trigger.positionTag}{/snippet}
          </Marker>
        {/if}
      {/snippet}
      {#snippet gutterBottom()}
        {#if xGround}
          <Marker
            kind="ground"
            axis="x"
            position={xFractionFromGround(xGround.groundPosition)}
            label={String(xGround.number)}
            name="Channel {xGround.number} ground (X axis)"
            color="var(--ss-color-ch{xGround.number})"
            disabled={xGround.availability?.enabled === false}
            disabledReason={xGround.availability?.reason}
            oncommit={(fraction) => windowActions.setChannelPosition(xGround.number, divisionsFromXFraction(fraction))}
            onreset={() => windowActions.setChannelPosition(xGround.number, 0)}
          />
        {/if}
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
  .peak-layer {
    position: absolute;
    inset: 0;
    cursor: crosshair;
  }
  .tag {
    position: absolute;
    padding: var(--ss-space-1) var(--ss-space-3);
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-secondary);
    background: var(--ss-color-bg-overlay);
    border-radius: var(--ss-radius-sm);
    pointer-events: none;
  }
  .top-left {
    inset-block-start: var(--ss-space-3);
    inset-inline-start: var(--ss-space-3);
    border: var(--ss-border-width) solid currentColor;
  }
  .top-right {
    inset-block-start: var(--ss-space-3);
    inset-inline-end: var(--ss-space-3);
  }
  .bottom-right {
    inset-block-end: var(--ss-space-3);
    inset-inline-end: var(--ss-space-3);
    border: var(--ss-border-width) solid currentColor;
  }
  .edge-right {
    inset-block-start: var(--ss-space-3);
    inset-inline-end: var(--ss-space-1);
    color: var(--ss-color-text-muted);
  }
</style>
