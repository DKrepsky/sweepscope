<script module lang="ts">
  import type { Snippet } from 'svelte';

  export interface ButtonProps {
    size?: 'sm' | 'md' | 'lg';
    /** Latched on state (toggle buttons), distinct from the transient press feedback. */
    pressed?: boolean;
    /** Status colour used by RunButton. */
    tone?: 'run' | 'stop';
    disabled?: boolean;
    disabledReason?: string;
    title?: string;
    ariaLabel?: string;
    onclick?: () => void;
    left?: Snippet;
    children?: Snippet;
    right?: Snippet;
  }
</script>

<script lang="ts">
  import Tooltip from '../tooltip/Tooltip.svelte';

  let {
    size = 'md',
    pressed,
    tone,
    disabled = false,
    disabledReason,
    title,
    ariaLabel,
    onclick,
    left,
    children,
    right,
  }: ButtonProps = $props();

  let down = $state(false);
  const tip = $derived(disabled ? disabledReason : title);
  const iconOnly = $derived(!children && !right);

  function activate() {
    if (!disabled) onclick?.();
  }

  function press() {
    if (!disabled) down = true;
  }

  function release() {
    down = false;
  }
</script>

{#snippet control()}
  <button
    type="button"
    class={['btn focus-ring', size, tone, { on: pressed, icon: iconOnly, disabled }]}
    aria-disabled={disabled ? 'true' : undefined}
    aria-pressed={pressed}
    aria-label={ariaLabel}
    data-pressed={down ? 'true' : undefined}
    onclick={activate}
    onpointerdown={press}
    onpointerup={release}
    onpointerleave={release}
    onpointercancel={release}
  >
    {@render left?.()}
    {@render children?.()}
    {@render right?.()}
  </button>
{/snippet}

{#if tip}
  <Tooltip text={tip}>{@render control()}</Tooltip>
{:else}
  {@render control()}
{/if}

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--ss-space-3);
    height: var(--ss-size-control-md);
    padding: 0 var(--ss-space-4);
    background: var(--ss-color-bg-raised);
    border: var(--ss-border-width) solid var(--ss-color-border);
    border-radius: var(--ss-radius-sm);
    font-size: var(--ss-font-size-sm);
    white-space: nowrap;
    transition: background var(--ss-motion-fast) var(--ss-motion-easing);
  }

  .btn:hover {
    background: var(--ss-color-bg-hover);
  }

  .btn:active,
  .btn[data-pressed] {
    background: var(--ss-color-bg-pressed);
  }

  .focus-ring:focus-visible {
    outline: var(--ss-border-width-focus) solid var(--ss-color-focus);
    outline-offset: var(--ss-border-width);
  }

  .sm {
    height: var(--ss-size-control-sm);
  }

  .lg {
    height: var(--ss-size-control-lg);
    padding: 0 var(--ss-space-5);
    font-weight: var(--ss-font-weight-bold);
    letter-spacing: var(--ss-letter-spacing-caps);
    text-transform: uppercase;
  }

  .icon {
    width: var(--ss-size-control-md);
    padding: 0;
  }

  .icon.sm {
    width: var(--ss-size-control-sm);
  }

  .on {
    background: var(--ss-color-bg-selected);
    border-color: var(--ss-color-accent);
    color: var(--ss-color-text-primary);
  }

  .run {
    color: var(--ss-color-status-run);
    border-color: var(--ss-color-status-run);
  }

  .stop {
    color: var(--ss-color-status-stop);
    border-color: var(--ss-color-status-stop);
  }

  .btn.disabled {
    color: var(--ss-color-text-disabled);
    background: var(--ss-color-bg-panel);
    border-color: var(--ss-color-border-subtle);
    cursor: not-allowed;
  }
</style>
