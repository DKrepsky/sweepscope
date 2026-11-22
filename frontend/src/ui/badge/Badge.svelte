<script module lang="ts">
  import type { Snippet } from 'svelte';

  export interface BadgeProps {
    kind: 'warning' | 'software' | 'offline' | 'error' | 'experimental';
    tooltip: string;
    children: Snippet;
  }
</script>

<script lang="ts">
  import Tooltip from '../tooltip/Tooltip.svelte';

  let { kind, tooltip, children }: BadgeProps = $props();

  const descriptionId = $props.id();
  const modifiers = { warning: 'warn dashed', software: 'sw', offline: 'off', error: 'err', experimental: 'exp' };
</script>

<Tooltip text={tooltip}>
  <span class={['badge', modifiers[kind]]} role="note" aria-describedby={descriptionId}>
    {@render children()}
    <span class="sr-only" id={descriptionId}>{tooltip}</span>
  </span>
</Tooltip>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    gap: var(--ss-space-1);
    height: 1rem;
    padding: 0 var(--ss-space-2);
    font-size: var(--ss-font-size-2xs);
    font-weight: var(--ss-font-weight-bold);
    letter-spacing: var(--ss-letter-spacing-caps);
    text-transform: uppercase;
    line-height: 1;
    white-space: nowrap;
    border: var(--ss-border-width) solid currentColor;
    border-radius: var(--ss-radius-sm);
  }

  .warn {
    color: var(--ss-color-status-warning);
  }

  .dashed {
    border-style: dashed;
  }

  .sw {
    color: var(--ss-color-status-info);
  }

  .off {
    color: var(--ss-color-status-offline);
    background: var(--ss-color-bg-raised);
  }

  .err {
    color: var(--ss-color-status-error);
    background: var(--ss-color-bg-raised);
  }

  .exp {
    color: var(--ss-color-text-muted);
    border-style: dotted;
  }

  .sr-only {
    position: absolute;
    width: var(--ss-border-width);
    height: var(--ss-border-width);
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }
</style>
