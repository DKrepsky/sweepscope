<script module lang="ts">
  import type { Snippet } from 'svelte';

  export interface TooltipProps {
    text: string;
    /** Hover delay in milliseconds; keyboard focus shows the tooltip at once. */
    delay?: number;
    children: Snippet;
  }
</script>

<script lang="ts">
  import { onDestroy } from 'svelte';

  let { text, delay = 500, children }: TooltipProps = $props();

  const id = $props.id();
  let visible = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  function cancelPending() {
    clearTimeout(timer);
    timer = undefined;
  }

  function show() {
    cancelPending();
    visible = true;
  }

  function hide() {
    cancelPending();
    visible = false;
  }

  function showAfterDelay() {
    cancelPending();
    timer = setTimeout(show, delay);
  }

  function hideOnEscape(event: KeyboardEvent) {
    if (visible && event.key === 'Escape') hide();
  }

  onDestroy(cancelPending);
</script>

<svelte:window onkeydown={hideOnEscape} />

<span
  class="host"
  role="presentation"
  onmouseenter={showAfterDelay}
  onmouseleave={hide}
  onfocusin={show}
  onfocusout={hide}
>
  {@render children()}
  {#if visible}
    <span class="tip" role="tooltip" {id}>{text}</span>
  {/if}
</span>

<style>
  .host {
    position: relative;
    display: inline-flex;
  }

  .tip {
    position: absolute;
    top: 100%;
    left: 0;
    z-index: var(--ss-z-popover);
    margin-top: var(--ss-space-2);
    width: max-content;
    max-width: 18rem;
    padding: var(--ss-space-3) var(--ss-space-4);
    background: var(--ss-color-bg-raised);
    border: var(--ss-border-width) solid var(--ss-color-border-strong);
    border-radius: var(--ss-radius-sm);
    box-shadow: var(--ss-shadow-popover);
    color: var(--ss-color-text-primary);
    font-size: var(--ss-font-size-xs);
    font-weight: var(--ss-font-weight-regular);
    letter-spacing: normal;
    text-transform: none;
    white-space: normal;
    pointer-events: none;
  }
</style>
