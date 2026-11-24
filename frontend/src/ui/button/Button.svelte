<script module lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  type OwnProps = 'class' | 'type' | 'children' | 'onclick' | 'disabled' | 'title';

  /** Extra attributes (for example the menu trigger ones) are forwarded to the `<button>`. */
  export interface ButtonProps extends Omit<HTMLButtonAttributes, OwnProps> {
    size?: 'sm' | 'md' | 'lg';
    /** Latched on state (toggle buttons), distinct from the transient press feedback. */
    pressed?: boolean;
    /** Status colour used by RunButton. */
    tone?: 'run' | 'stop';
    disabled?: boolean;
    disabledReason?: string;
    /** Work in progress: shows a spinner on the right and sets `aria-busy`. The size does not change. */
    busy?: boolean;
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
    busy = false,
    title,
    ariaLabel,
    onclick,
    left,
    children,
    right,
    ...rest
  }: ButtonProps = $props();

  let down = $state(false);
  const tip = $derived(disabled ? disabledReason : title);
  const iconOnly = $derived(!children && !right && !busy);

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
    {...rest}
    type="button"
    class={['btn focus-ring', size, tone, { on: pressed, icon: iconOnly, disabled }]}
    aria-disabled={disabled ? 'true' : undefined}
    aria-pressed={pressed}
    aria-busy={busy ? 'true' : undefined}
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
    {#if busy}
      <span class="spinner" aria-hidden="true"></span>
      <span class="dots" aria-hidden="true">...</span>
    {:else}
      {@render right?.()}
    {/if}
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
    border: var(--ss-border-width) solid var(--ss-color-border-control);
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

  .spinner {
    flex: none;
    width: var(--ss-size-icon);
    height: var(--ss-size-icon);
    border: var(--ss-border-width-focus) solid currentColor;
    border-inline-end-color: transparent;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .dots {
    display: none;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .spinner {
      display: none;
    }

    .dots {
      display: inline;
      width: var(--ss-size-icon);
    }
  }

  .btn.disabled {
    color: var(--ss-color-text-disabled);
    background: var(--ss-color-bg-panel);
    border-color: var(--ss-color-border-subtle);
    cursor: not-allowed;
  }
</style>
