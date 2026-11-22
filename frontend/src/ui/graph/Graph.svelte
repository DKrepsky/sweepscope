<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Canvas2dRenderer } from './canvas-2d-renderer';
  import { watchDevicePixelRatio } from './device-pixel-ratio';
  import { deviceExtent } from './graticule';
  import { EMPTY_FRAME } from './graph-frame';
  import type { GraphFrame, TraceRenderer } from './graph-frame';

  interface Layers {
    graticule: HTMLCanvasElement;
    traces: HTMLCanvasElement;
  }

  interface Props {
    frame?: GraphFrame;
    summary: string;
    dots?: boolean;
    createRenderer?: (layers: Layers) => TraceRenderer;
    overlay?: Snippet;
    gutterLeft?: Snippet;
    gutterRight?: Snippet;
    gutterTop?: Snippet;
  }

  const canvasRenderer = ({ graticule, traces }: Layers): TraceRenderer =>
    new Canvas2dRenderer(graticule.getContext('2d')!, traces.getContext('2d')!);

  let {
    frame = EMPTY_FRAME,
    summary,
    dots = false,
    createRenderer = canvasRenderer,
    overlay,
    gutterLeft,
    gutterRight,
    gutterTop,
  }: Props = $props();

  let screen: HTMLElement;
  let graticuleCanvas: HTMLCanvasElement;
  let tracesCanvas: HTMLCanvasElement;
  let renderer = $state<TraceRenderer | null>(null);
  let box = $state({ width: 0, height: 0 });
  let ratio = $state(window.devicePixelRatio || 1);

  $effect(() => {
    const created = createRenderer({ graticule: graticuleCanvas, traces: tracesCanvas });
    renderer = created;
    return () => {
      created.dispose();
      renderer = null;
    };
  });

  $effect(() => {
    const measure = () => (box = { width: screen.clientWidth, height: screen.clientHeight });
    measure();
    if (typeof ResizeObserver === 'undefined') return;
    const observer = new ResizeObserver(measure);
    observer.observe(screen);
    return () => observer.disconnect();
  });

  $effect(() => watchDevicePixelRatio(window, (next) => (ratio = next)));

  /** Assigning width or height clears a canvas even for the same value, which would erase the cached graticule. */
  function resizeBitmap(canvas: HTMLCanvasElement, css: { width: number; height: number }, scale: number) {
    const width = deviceExtent(css.width, scale);
    const height = deviceExtent(css.height, scale);
    if (canvas.width !== width) canvas.width = width;
    if (canvas.height !== height) canvas.height = height;
  }

  $effect(() => {
    if (!renderer || box.width === 0 || box.height === 0) return;
    for (const canvas of [graticuleCanvas, tracesCanvas]) resizeBitmap(canvas, box, ratio);
    renderer.draw({ ...frame, dots: dots || frame.dots }, { ...box, ratio });
  });
</script>

<div class="graph" data-state={frame.state}>
  <div class="screen" bind:this={screen}>
    <div class="layers" role="img" aria-label={summary}>
      <canvas bind:this={graticuleCanvas} class="layer"></canvas>
      <canvas bind:this={tracesCanvas} class="layer"></canvas>
    </div>
    {#if overlay}<div class="overlay">{@render overlay()}</div>{/if}
  </div>
  {#if gutterLeft}<div class="gutter left">{@render gutterLeft()}</div>{/if}
  {#if gutterRight}<div class="gutter right">{@render gutterRight()}</div>{/if}
  {#if gutterTop}<div class="gutter top">{@render gutterTop()}</div>{/if}
</div>

<style>
  .graph {
    position: relative;
    inline-size: 100%;
    block-size: 100%;
  }

  .screen {
    position: relative;
    inline-size: 100%;
    block-size: 100%;
    background: var(--ss-color-bg-scope);
  }

  .layers {
    position: absolute;
    inset: 0;
  }

  .layer {
    position: absolute;
    inset: 0;
    inline-size: 100%;
    block-size: 100%;
  }

  .overlay {
    position: absolute;
    inset: 0;
  }

  .gutter {
    position: absolute;
    z-index: var(--ss-z-markers);
  }

  .gutter.left {
    inset-block: 0;
    inset-inline-end: 100%;
    inline-size: calc(var(--ss-space-7) + var(--ss-space-4));
  }

  .gutter.right {
    inset-block: 0;
    inset-inline-start: 100%;
    inline-size: calc(var(--ss-space-7) + var(--ss-space-4));
  }

  .gutter.top {
    inset-inline: 0;
    inset-block-end: 100%;
    block-size: calc(var(--ss-space-6) + var(--ss-space-2));
  }
</style>
