<script lang="ts">
  import type { Snippet } from 'svelte';
  import { axisOf, nudgedPosition, positionFromPointer } from './marker-geometry';
  import type { MarkerKind } from './marker-geometry';

  interface Props {
    kind: MarkerKind;
    position: number;
    label: string;
    name: string;
    color?: string;
    pending?: boolean;
    disabled?: boolean;
    disabledReason?: string;
    step?: number;
    tag?: Snippet;
    ondrag?: (position: number) => void;
    oncommit?: (position: number) => void;
    ondelete?: () => void;
    onreset?: () => void;
  }

  let {
    kind,
    position,
    label,
    name,
    color,
    pending = false,
    disabled = false,
    disabledReason,
    step = 0.0125,
    tag,
    ondrag,
    oncommit,
    ondelete,
    onreset,
  }: Props = $props();

  const axis = $derived(axisOf(kind));
  let dragPosition = $state<number | null>(null);
  let lastPosition: number | null = null;
  const shown = $derived(dragPosition ?? position);
  const placement = $derived(axis === 'vertical' ? `top: ${(1 - shown) * 100}%` : `left: ${shown * 100}%`);

  function moveTo(next: number | null) {
    if (next === null) return;
    dragPosition = next;
    lastPosition = next;
    ondrag?.(next);
  }

  function onpointerdown(event: PointerEvent) {
    if (disabled) return;
    (event.currentTarget as HTMLElement).setPointerCapture?.(event.pointerId);
    moveTo(trackPosition(event));
  }

  function onpointermove(event: PointerEvent) {
    if (dragPosition === null) return;
    moveTo(trackPosition(event));
  }

  function trackPosition(event: PointerEvent): number | null {
    const track = (event.currentTarget as HTMLElement).parentElement!.getBoundingClientRect();
    return positionFromPointer(axis, { x: event.clientX, y: event.clientY }, track);
  }

  function onpointerup() {
    if (dragPosition === null) return;
    const last = lastPosition;
    dragPosition = null;
    lastPosition = null;
    if (last !== null) oncommit?.(last);
  }

  function onkeydown(event: KeyboardEvent) {
    if (disabled) return;
    if (event.key === 'Delete') return ondelete?.();
    const next = nudgedPosition(axis, event, position, step);
    if (next === null) return;
    event.preventDefault();
    ondrag?.(next);
    oncommit?.(next);
  }
</script>

<div
  class="marker {kind}"
  class:pending
  class:dragging={dragPosition !== null}
  class:disabled
  role="slider"
  tabindex="0"
  aria-label={name}
  aria-orientation={axis}
  aria-valuemin="0"
  aria-valuemax="1"
  aria-valuenow={position}
  aria-disabled={disabled}
  title={disabled ? disabledReason : undefined}
  style="{placement}; {color ? `--marker-colour: ${color}` : ''}"
  {onpointerdown}
  {onpointermove}
  {onpointerup}
  onpointercancel={onpointerup}
  ondblclick={() => !disabled && onreset?.()}
  {onkeydown}
>
  <span class="shape">{label}</span>
  {#if pending}
    <span class="tag">...</span>
  {:else if tag}
    <span class="tag">{@render tag()}</span>
  {/if}
</div>

<style>
  .marker {
    --marker-colour: var(--ss-color-trigger);
    position: absolute;
    display: grid;
    padding: var(--ss-space-4);
    margin: calc(-1 * var(--ss-space-4));
    cursor: ns-resize;
    touch-action: none;
    outline: none;
    transform: translateY(-50%);
  }

  .marker.trigger-position {
    cursor: ew-resize;
    transform: translateX(-50%);
  }

  .marker.ground {
    inset-inline-end: 0;
  }

  .marker.trigger-level {
    inset-inline-start: 0;
  }

  .marker.trigger-position {
    inset-block-end: 0;
  }

  .marker.disabled {
    cursor: default;
  }

  .shape {
    display: grid;
    place-items: center;
    inline-size: var(--ss-size-marker);
    block-size: var(--ss-size-marker);
    background: var(--marker-colour);
    color: var(--ss-color-text-on-accent);
    font: var(--ss-font-weight-bold) var(--ss-font-size-2xs) / 1 var(--ss-font-mono);
  }

  .ground .shape {
    clip-path: polygon(0 0, 70% 0, 100% 50%, 70% 100%, 0 100%);
  }

  .trigger-level .shape {
    clip-path: polygon(30% 0, 100% 0, 100% 100%, 30% 100%, 0 50%);
    padding-inline-start: var(--ss-space-1);
  }

  .trigger-position .shape {
    clip-path: polygon(0 0, 100% 0, 100% 60%, 50% 100%, 0 60%);
    padding-block-end: var(--ss-space-1);
  }

  .marker:hover .shape {
    filter: brightness(1.15);
  }

  .marker:focus-visible {
    filter: drop-shadow(var(--ss-border-width-focus) 0 0 var(--ss-color-focus))
      drop-shadow(calc(-1 * var(--ss-border-width-focus)) 0 0 var(--ss-color-focus))
      drop-shadow(0 var(--ss-border-width-focus) 0 var(--ss-color-focus))
      drop-shadow(0 calc(-1 * var(--ss-border-width-focus)) 0 var(--ss-color-focus));
  }

  .dragging .shape {
    opacity: 0.6;
  }

  .pending .shape,
  .dragging .shape {
    clip-path: none;
    outline: var(--ss-border-width) dashed var(--ss-color-text-primary);
  }

  .marker.disabled .shape {
    opacity: 0.4;
  }

  .tag {
    position: absolute;
    inset-block-start: 50%;
    transform: translateY(-50%);
    font: var(--ss-font-size-xs) / 1 var(--ss-font-mono);
    padding: var(--ss-space-1) var(--ss-space-2);
    background: var(--ss-color-bg-overlay);
    border: var(--ss-border-width) solid currentColor;
    border-radius: var(--ss-radius-sm);
    color: var(--marker-colour);
    white-space: nowrap;
    pointer-events: none;
  }

  .ground .tag {
    inset-inline-end: 100%;
  }

  .trigger-level .tag {
    inset-inline-start: 100%;
  }

  .trigger-position .tag {
    inset-inline-start: 100%;
    inset-block-start: 0;
    transform: none;
  }
</style>
