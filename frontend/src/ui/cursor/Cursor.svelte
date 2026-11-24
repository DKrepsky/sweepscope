<script lang="ts">
  import type { Snippet } from 'svelte';
  import { positionFromPointer, steppedPosition, type CursorAxis } from './cursor-geometry';

  interface Props {
    axis: CursorAxis;
    /** x from the left, y from the top, 0 to 1 of the screen. */
    position: number;
    label: string;
    valueText: string;
    onmove: (position: number) => void;
    onremove: () => void;
    disabled?: boolean;
    disabledReason?: string;
    tag?: Snippet;
  }

  const { axis, position, label, valueText, onmove, onremove, disabled = false, disabledReason, tag }: Props = $props();

  let dragging = $state(false);
  let dragStart = 0;
  let shown = $state<number | null>(null);
  const current = $derived(shown ?? position);
  const placement = $derived(axis === 'x' ? `left: ${current * 100}%` : `top: ${current * 100}%`);

  $effect(() => {
    void position;
    shown = null;
  });

  function moveTo(next: number | null) {
    if (next === null) return;
    shown = next;
    onmove(next);
  }

  function onpointerdown(event: PointerEvent) {
    if (disabled) return;
    (event.currentTarget as HTMLElement).setPointerCapture?.(event.pointerId);
    dragging = true;
    dragStart = current;
    moveTo(pointerPosition(event));
  }

  function onpointermove(event: PointerEvent) {
    if (dragging) moveTo(pointerPosition(event));
  }

  function pointerPosition(event: PointerEvent): number | null {
    const track = (event.currentTarget as HTMLElement).parentElement!.getBoundingClientRect();
    return positionFromPointer(axis, { x: event.clientX, y: event.clientY }, track);
  }

  function endDrag() {
    dragging = false;
  }

  function onkeydown(event: KeyboardEvent) {
    if (disabled) return;
    if (event.key === 'Delete') return onremove();
    if (event.key === 'Escape' && dragging) {
      endDrag();
      return moveTo(dragStart);
    }
    const next = steppedPosition(axis, event, current);
    if (next === null) return;
    event.preventDefault();
    moveTo(next);
  }
</script>

<div
  class="cursor {axis}"
  class:dragging
  class:disabled
  role="slider"
  tabindex="0"
  aria-label={label}
  aria-orientation={axis === 'x' ? 'horizontal' : 'vertical'}
  aria-valuemin="0"
  aria-valuemax="1"
  aria-valuenow={current}
  aria-valuetext={valueText}
  aria-disabled={disabled}
  title={disabled ? disabledReason : undefined}
  style={placement}
  {onpointerdown}
  {onpointermove}
  onpointerup={endDrag}
  onpointercancel={endDrag}
  ondblclick={() => !disabled && onremove()}
  {onkeydown}
>
  <span class="line"></span>
  {#if tag}
    <span class="tag">{@render tag()}</span>
  {/if}
</div>

<style>
  .cursor {
    --cursor-hit: var(--ss-space-4);
    --cursor-dash: 0.375rem;
    --cursor-gap: 0.25rem;
    position: absolute;
    display: grid;
    place-items: center;
    color: var(--ss-color-cursor);
    pointer-events: auto;
    touch-action: none;
    outline: none;
  }
  .x {
    inset-block: 0;
    inline-size: calc(2 * var(--cursor-hit));
    transform: translateX(-50%);
    cursor: ew-resize;
  }
  .y {
    inset-inline: 0;
    block-size: calc(2 * var(--cursor-hit));
    transform: translateY(-50%);
    cursor: ns-resize;
  }
  .disabled {
    cursor: default;
  }
  .line {
    pointer-events: none;
  }
  .x .line {
    block-size: 100%;
    inline-size: var(--ss-border-width);
    background: repeating-linear-gradient(
      to bottom,
      currentcolor 0 var(--cursor-dash),
      transparent var(--cursor-dash) calc(var(--cursor-dash) + var(--cursor-gap))
    );
  }
  .y .line {
    inline-size: 100%;
    block-size: var(--ss-border-width);
    background: repeating-linear-gradient(
      to right,
      currentcolor 0 var(--cursor-dash),
      transparent var(--cursor-dash) calc(var(--cursor-dash) + var(--cursor-gap))
    );
  }
  .cursor:hover .line,
  .dragging .line {
    filter: brightness(1.3);
  }
  .cursor:focus-visible .line {
    outline: var(--ss-border-width-focus) solid var(--ss-color-focus);
    outline-offset: var(--ss-border-width);
  }
  .tag {
    position: absolute;
    z-index: var(--ss-z-markers);
    padding: var(--ss-space-1) var(--ss-space-2);
    font: var(--ss-font-size-xs) / 1 var(--ss-font-mono);
    white-space: nowrap;
    background: var(--ss-color-bg-overlay);
    border: var(--ss-border-width) solid currentcolor;
    border-radius: var(--ss-radius-sm);
    pointer-events: none;
  }
  .x .tag {
    inset-block-end: var(--ss-space-3);
  }
  .y .tag {
    inset-inline-start: var(--ss-space-3);
  }
  .disabled .line {
    opacity: 0.4;
  }
</style>
