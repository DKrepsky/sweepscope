<script lang="ts">
  import Graph from '../../ui/graph/Graph.svelte';
  import Marker from '../../ui/marker/Marker.svelte';
  import { StatusBar } from '../../ui/status-bar';
  import { StatusChip } from '../../ui/status-chip';
  import { emptyFrame, frozenFrame, gapFrame, liveFrame } from './mock-data';
  import Row from './Row.svelte';
  import Section from './Section.svelte';

  const noop = () => {};

  const POINTS = 400;
  const trace = (shape: (i: number) => number) => Array.from({ length: POINTS }, (_, i) => shape(i));
  const sineAt = (phase: number, amplitude: number) => 4 - amplitude * Math.sin(phase * 2 * Math.PI);

  const tickedFrame: typeof liveFrame = {
    ...liveFrame,
    channels: liveFrame.channels.slice(0, 1),
    ticksX: [0, 0.25, 0.5, 0.75, 1].map((at) => ({ at, text: `${(at * 2).toFixed(1)} ms` })),
    ticksY: [0, 0.25, 0.5, 0.75, 1].map((at) => ({ at, text: `${((at - 0.5) * 8).toFixed(0)} V` })),
  };

  const xyFrame: typeof liveFrame = {
    state: 'live',
    channels: [],
    xy: {
      x: trace((i) => sineAt(i / POINTS, 3)),
      y: trace((i) => sineAt((i / POINTS) * 2 + 0.25, 3)),
      xChannel: 1,
      yChannel: 2,
    },
  };

  const brokenFrame: typeof liveFrame = {
    state: 'live',
    channels: [{ channel: 1, samples: trace((i) => (i > 150 && i < 190 ? Number.NaN : sineAt((i / POINTS) * 3, 2))) }],
  };

  let persistFrame = $state<typeof liveFrame>({ state: 'live', channels: [] });
  $effect(() => {
    let tick = 0;
    const next = () => {
      tick += 1;
      const glitch = tick % 9 === 0;
      const samples = trace((i) => sineAt((i / POINTS) * 3 + tick * 0.01, 2.5) + (glitch && i === 200 ? -1.5 : 0));
      persistFrame = {
        state: 'live',
        channels: [{ channel: 1, samples }],
        persistence: { decayMs: 1000, epoch: 0, now: () => performance.now() },
      };
    };
    next();
    const timer = setInterval(next, 100);
    return () => clearInterval(timer);
  });
</script>

{#snippet screen(caption: string, frame: typeof liveFrame, summary: string, overlay?: import('svelte').Snippet)}
  <figure class="frame">
    <figcaption>{caption}</figcaption>
    <div class="screen">
      <Graph {frame} {summary} {overlay} />
    </div>
  </figure>
{/snippet}

{#snippet gap()}
  <div class="gap-band"></div>
  <span class="overlay-tag warning">GAP &#8805; 3 chunks lost</span>
{/snippet}

{#snippet disconnected()}
  <span class="overlay-tag disconnected">DISCONNECTED &middot; last frame 14:02:31</span>
{/snippet}

<Section component="Graph" note="Illustrative waveforms. Frozen traces are drawn at 40 % opacity.">
  <div class="frames">
    {@render screen('Live, four channels', liveFrame, 'Illustrative: four channels')}
    {@render screen('Frozen', frozenFrame, 'Illustrative: frozen CH1')}
    {@render screen('Empty', emptyFrame, 'Illustrative: no waveform')}
    {@render screen('Roll mode with data gap', gapFrame, 'Illustrative: CH1 with a gap', gap)}
    {@render screen('Disconnected: last frame frozen', frozenFrame, 'Illustrative: disconnected', disconnected)}
    {@render screen('Tick labels inside the frame', tickedFrame, 'Illustrative: CH1 with tick labels')}
    {@render screen('XY, square divisions, Y channel colour', xyFrame, 'Illustrative: XY figure')}
    {@render screen('NaN samples break the line', brokenFrame, 'Illustrative: CH1 with missing samples')}
    {@render screen('Persistence 1 s, an occasional glitch', persistFrame, 'Illustrative: persistence ghosts')}
  </div>
</Section>

<Section
  component="Marker"
  note="Idle, selected colour, pending and disabled. X ground marker in the bottom gutter, peak marker on the screen. Keyboard: Tab, then Alt+Up/Down (peak: Left/Right, Shift for ten steps)."
>
  <Row>
    <div class="marker-stage">
      <Graph frame={emptyFrame} summary="Illustrative: markers">
        {#snippet gutterLeft()}
          <Marker kind="ground" position={0.5} label="1" name="CH1 ground" color="var(--ss-color-ch1)" ondrag={noop} />
          <Marker kind="ground" position={0.3} label="2" name="CH2 ground" color="var(--ss-color-ch2)" pending />
        {/snippet}
        {#snippet gutterRight()}
          <Marker kind="trigger-level" position={0.6} label="T" name="Trigger level" ondrag={noop}>
            {#snippet tag()}1.20 V{/snippet}
          </Marker>
          <Marker
            kind="trigger-level"
            position={0.2}
            label="T"
            name="Trigger level, disabled"
            disabled
            disabledReason="Trigger not available: offline"
          />
        {/snippet}
        {#snippet gutterTop()}
          <Marker kind="trigger-position" position={0.5} label="T" name="Trigger position" />
        {/snippet}
        {#snippet gutterBottom()}
          <Marker kind="ground" axis="x" position={0.45} label="1" name="CH1 X ground" color="var(--ss-color-ch1)" />
        {/snippet}
        {#snippet overlay()}
          <Marker
            kind="peak"
            position={0.3}
            level={0.8}
            label=""
            name="Spectrum peak"
            color="var(--ss-color-cursor)"
            step={0.01}
            valueText="1.001 kHz, -6.9 dBV"
          >
            {#snippet tag()}1.001 kHz &nbsp; &minus;6.9 dBV{/snippet}
          </Marker>
        {/snippet}
      </Graph>
    </div>
  </Row>
</Section>

<Section component="StatusBar" note="Normal and warning items.">
  <StatusBar
    items={[
      { id: 'connection', priority: 0, content: connection },
      { id: 'rate', priority: 1, content: rate },
      { id: 'note', priority: 2, warning: true, content: warning },
    ]}
  />
</Section>

{#snippet connection()}<StatusChip kind="run">Ready</StatusChip>{/snippet}
{#snippet rate()}<span class="ss-num">Sample rate 10 MSa/s</span>{/snippet}
{#snippet warning()}Aliasing risk (illustrative){/snippet}

<style>
  .frames {
    display: flex;
    flex-wrap: wrap;
    gap: var(--ss-space-6);
  }

  .frame {
    display: grid;
    gap: var(--ss-space-2);
  }

  figcaption {
    color: var(--ss-color-text-secondary);
    font-size: var(--ss-font-size-xs);
  }

  .screen {
    inline-size: 26rem;
    block-size: 11rem;
    border: var(--ss-border-width) solid var(--ss-color-grid-frame);
  }

  .marker-stage {
    position: relative;
    inline-size: 26rem;
    block-size: 11rem;
    margin: var(--ss-space-6) var(--ss-space-7);
  }

  .gap-band {
    position: absolute;
    inset-block: 0;
    inset-inline-start: 35%;
    inline-size: 12%;
    background: var(--ss-color-status-warning);
    opacity: 0.14;
    border-inline: var(--ss-border-width) dashed var(--ss-color-status-warning);
  }

  .overlay-tag {
    position: absolute;
    padding: var(--ss-space-2) var(--ss-space-4);
    background: var(--ss-color-bg-overlay);
    font-size: var(--ss-font-size-xs);
  }

  .overlay-tag.warning {
    inset-block-start: var(--ss-space-3);
    inset-inline-start: 30%;
    color: var(--ss-color-status-warning);
  }

  .overlay-tag.disconnected {
    inset-block-start: 50%;
    inset-inline-start: 50%;
    transform: translate(-50%, -50%);
    color: var(--ss-color-status-disconnected);
    font-size: var(--ss-font-size-md);
  }
</style>
