<script module lang="ts">
  import type { Snippet } from 'svelte';

  export type StatusChipKind =
    'run' | 'stop' | 'armed' | 'triggered' | 'auto' | 'disconnected' | 'error' | 'warning' | 'info' | 'offline' | 'ok';

  export interface StatusChipProps {
    kind: StatusChipKind;
    filled?: boolean;
    /** Inactive state: 40 % opacity, same box. */
    dim?: boolean;
    children: Snippet;
    /** Replaces the default glyph of the kind. */
    left?: Snippet;
  }
</script>

<script lang="ts">
  import type { IconName } from '../../design-system';
  import Icon from '../icon/Icon.svelte';

  let { kind, filled = false, dim = false, children, left }: StatusChipProps = $props();

  const glyphs: Record<StatusChipKind, IconName | 'dot'> = {
    run: 'dot',
    stop: 'stop',
    armed: 'dot',
    triggered: 'rising-edge',
    auto: 'autoset',
    disconnected: 'disconnected',
    error: 'warning',
    warning: 'warning',
    info: 'info',
    offline: 'minus',
    ok: 'dot',
  };
  const glyph = $derived(glyphs[kind]);
</script>

<span class={['chip', kind, { filled, dim }]} aria-live="polite" aria-atomic="true">
  <span class="glyph" data-glyph={glyph}>
    {#if left}
      {@render left()}
    {:else if glyph === 'dot'}
      <span class="dot"></span>
    {:else}
      <Icon name={glyph} />
    {/if}
  </span>
  <span class="text">{@render children()}</span>
</span>

<style>
  .chip {
    display: inline-flex;
    align-items: center;
    gap: var(--ss-space-3);
    height: var(--ss-size-control-md);
    padding: 0 var(--ss-space-4);
    background: var(--ss-color-bg-inset);
    border: var(--ss-border-width) solid currentColor;
    border-radius: var(--ss-radius-sm);
    font-size: var(--ss-font-size-xs);
    font-weight: var(--ss-font-weight-bold);
    letter-spacing: var(--ss-letter-spacing-caps);
    text-transform: uppercase;
    white-space: nowrap;
  }

  .glyph {
    display: inline-flex;
  }

  .dot {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
    background: currentColor;
  }

  .run {
    color: var(--ss-color-status-run);
  }

  .ok {
    color: var(--ss-color-status-ok);
  }

  .stop {
    color: var(--ss-color-status-stop);
  }

  .armed {
    color: var(--ss-color-status-armed);
  }

  .triggered {
    color: var(--ss-color-status-triggered);
  }

  .auto {
    color: var(--ss-color-status-auto);
  }

  .disconnected {
    color: var(--ss-color-status-disconnected);
  }

  .error {
    color: var(--ss-color-status-error);
  }

  .warning {
    color: var(--ss-color-status-warning);
  }

  .info {
    color: var(--ss-color-status-info);
  }

  .offline {
    color: var(--ss-color-status-offline);
  }

  .filled {
    background: currentColor;
  }

  .filled > * {
    color: var(--ss-color-text-on-accent);
  }

  .dim {
    opacity: 0.4;
  }
</style>
