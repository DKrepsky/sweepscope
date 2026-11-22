<script module lang="ts">
  import type { Snippet } from 'svelte';

  export interface StepperProps {
    /** Display string, already formatted by the caller. */
    value: string;
    unit?: string;
    onstep: (dir: 1 | -1, coarse?: boolean) => void;
    onreset?: () => void;
    atMin?: boolean;
    atMax?: boolean;
    off?: boolean;
    /** Spoken value with unit, for example "500 millivolts per division". */
    valueText: string;
    ariaLabel: string;
    badge?: Snippet;
    unitSnippet?: Snippet;
  }
</script>

<script lang="ts">
  import Icon from '../icon/Icon.svelte';

  let {
    value,
    unit,
    onstep,
    onreset,
    atMin = false,
    atMax = false,
    off = false,
    valueText,
    ariaLabel,
    badge,
    unitSnippet,
  }: StepperProps = $props();

  const blocked = (dir: 1 | -1) => off || (dir === 1 ? atMax : atMin);

  function step(dir: 1 | -1, coarse = false) {
    if (blocked(dir)) return;
    if (coarse) onstep(dir, true);
    else onstep(dir);
  }

  const keySteps: Record<string, [1 | -1, boolean]> = {
    ArrowUp: [1, false],
    ArrowDown: [-1, false],
    PageUp: [1, true],
    PageDown: [-1, true],
  };

  function onkeydown(event: KeyboardEvent) {
    const entry = keySteps[event.key];
    if (!entry) return;
    event.preventDefault();
    step(entry[0], entry[1]);
  }

  function onwheel(event: WheelEvent) {
    if (event.deltaY === 0) return;
    event.preventDefault();
    step(event.deltaY < 0 ? 1 : -1);
  }

  function reset() {
    if (!off) onreset?.();
  }
</script>

<span class="stepper" role="presentation" {onwheel}>
  <span class={['step', { off }]}>
    <button
      type="button"
      class="focus-ring"
      tabindex="-1"
      aria-label="Decrease"
      aria-disabled={blocked(-1) ? 'true' : undefined}
      onclick={() => step(-1)}
    >
      <Icon name="minus" />
    </button>
    <span
      class="val focus-ring"
      role="spinbutton"
      tabindex="0"
      aria-label={ariaLabel}
      aria-valuetext={valueText}
      aria-disabled={off ? 'true' : undefined}
      {onkeydown}
      ondblclick={reset}
    >
      <span class="ss-num">{value}</span>
      {#if unitSnippet}{@render unitSnippet()}{:else if unit}<small>{unit}</small>{/if}
    </span>
    <button
      type="button"
      class="focus-ring"
      tabindex="-1"
      aria-label="Increase"
      aria-disabled={blocked(1) ? 'true' : undefined}
      onclick={() => step(1)}
    >
      <Icon name="plus" />
    </button>
  </span>
  {@render badge?.()}
</span>

<style>
  .stepper {
    display: inline-flex;
    align-items: center;
    gap: var(--ss-space-3);
  }

  .step {
    display: inline-flex;
    align-items: stretch;
    height: var(--ss-size-control-md);
    border: var(--ss-border-width) solid var(--ss-color-border);
    border-radius: var(--ss-radius-sm);
    background: var(--ss-color-bg-inset);
  }

  .off {
    opacity: 0.45;
  }

  button {
    display: grid;
    place-items: center;
    width: var(--ss-size-control-md);
    background: var(--ss-color-bg-raised);
    color: var(--ss-color-text-secondary);
  }

  button:first-child {
    border-right: var(--ss-border-width) solid var(--ss-color-border);
  }

  button:last-of-type {
    border-left: var(--ss-border-width) solid var(--ss-color-border);
  }

  button:hover {
    background: var(--ss-color-bg-hover);
  }

  button[aria-disabled='true'] {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .focus-ring:focus-visible {
    outline: var(--ss-border-width-focus) solid var(--ss-color-focus);
    outline-offset: calc(-1 * var(--ss-border-width-focus));
  }

  .val {
    display: flex;
    align-items: baseline;
    justify-content: flex-end;
    gap: var(--ss-space-2);
    min-width: 5.75rem;
    padding: 0 var(--ss-space-3);
    align-self: center;
    font-size: var(--ss-font-size-md);
    color: var(--ss-color-text-primary);
  }

  small {
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-muted);
  }
</style>
