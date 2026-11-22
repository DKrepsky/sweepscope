import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import Graph from './Graph.svelte';
import type { GraphFrame, PixelSize, TraceRenderer } from './graph-frame';

const frame: GraphFrame = { state: 'live', channels: [{ channel: 1, samples: [1, 2, 3] }] };

function fakeRenderer() {
  const draws: { frame: GraphFrame; size: PixelSize }[] = [];
  const renderer: TraceRenderer = { draw: (f, s) => void draws.push({ frame: f, size: s }), dispose: vi.fn() };
  return { renderer, draws };
}

function stubScreenSize(width: number, height: number) {
  vi.spyOn(HTMLElement.prototype, 'clientWidth', 'get').mockReturnValue(width);
  vi.spyOn(HTMLElement.prototype, 'clientHeight', 'get').mockReturnValue(height);
}

beforeEach(() => {
  window.matchMedia = (() => ({
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
  })) as unknown as typeof window.matchMedia;
});

afterEach(() => vi.restoreAllMocks());

describe('Graph', () => {
  it('given a summary, when rendered, then the canvases are exposed as one image with that label', () => {
    render(Graph, { summary: 'CH1 1 V/div, sine 1 kHz', createRenderer: () => fakeRenderer().renderer });
    expect(screen.getByRole('img', { name: 'CH1 1 V/div, sine 1 kHz' })).toBeInTheDocument();
  });

  it('given a measured screen, when a frame is passed, then the renderer draws it at the screen size and ratio', async () => {
    stubScreenSize(500, 400);
    const { renderer, draws } = fakeRenderer();
    render(Graph, { summary: 's', frame, createRenderer: () => renderer });
    await vi.waitFor(() => expect(draws).toHaveLength(1));
    expect(draws[0]!.size).toEqual({ width: 500, height: 400, ratio: window.devicePixelRatio || 1 });
    expect(draws[0]!.frame.channels).toEqual(frame.channels);
  });

  it('given the dots prop, when drawn, then the frame handed to the renderer has dots set', async () => {
    stubScreenSize(500, 400);
    const { renderer, draws } = fakeRenderer();
    render(Graph, { summary: 's', frame, dots: true, createRenderer: () => renderer });
    await vi.waitFor(() => expect(draws[0]?.frame.dots).toBe(true));
  });

  it('given a screen of unknown size, when rendered, then nothing is drawn', async () => {
    const { renderer, draws } = fakeRenderer();
    render(Graph, { summary: 's', frame, createRenderer: () => renderer });
    await Promise.resolve();
    expect(draws).toEqual([]);
  });

  it('given a drawn graph, when a new frame arrives at the same size, then the bitmaps are not reassigned and so not cleared', async () => {
    stubScreenSize(500, 400);
    const widthSet = vi.spyOn(HTMLCanvasElement.prototype, 'width', 'set');
    const { renderer, draws } = fakeRenderer();
    const { rerender } = render(Graph, { summary: 's', frame, createRenderer: () => renderer });
    await vi.waitFor(() => expect(draws).toHaveLength(1));
    const assignmentsAfterFirstDraw = widthSet.mock.calls.length;
    await rerender({ summary: 's', frame: { ...frame, state: 'frozen' } });
    await vi.waitFor(() => expect(draws).toHaveLength(2));
    expect(widthSet.mock.calls.length).toBe(assignmentsAfterFirstDraw);
  });

  it('given an unmounted graph, when it is destroyed, then the renderer is disposed', () => {
    const { renderer } = fakeRenderer();
    const { unmount } = render(Graph, { summary: 's', createRenderer: () => renderer });
    unmount();
    expect(renderer.dispose).toHaveBeenCalledOnce();
  });

  it('given gutter and overlay snippets, when rendered, then each content is present outside the labelled image', () => {
    const text = (label: string) => createRawSnippet(() => ({ render: () => `<button>${label}</button>` }));
    render(Graph, {
      summary: 's',
      createRenderer: () => fakeRenderer().renderer,
      gutterLeft: text('left'),
      gutterRight: text('right'),
      gutterTop: text('top'),
      overlay: text('overlay'),
    });
    const image = screen.getByRole('img');
    for (const name of ['left', 'right', 'top', 'overlay']) {
      expect(image).not.toContainElement(screen.getByRole('button', { name }));
    }
  });
});
