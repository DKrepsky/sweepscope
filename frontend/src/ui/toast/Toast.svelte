<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from '../icon/Icon.svelte';

  interface Props {
    kind: 'info' | 'warning' | 'error';
    text: Snippet;
    timeoutMs?: number;
    ondismiss: () => void;
  }

  const { kind, text, timeoutMs, ondismiss }: Props = $props();

  const defaultTimeoutMs = { info: 4000, warning: 8000, error: 0 } as const;

  let paused = $state(false);

  $effect(() => {
    if (kind === 'error' || paused) return;
    const timer = setTimeout(ondismiss, timeoutMs ?? defaultTimeoutMs[kind]);
    return () => clearTimeout(timer);
  });
</script>

<!-- Pointer and focus only pause the timer; the toast never takes focus itself. -->
<div
  class="toast {kind}"
  role={kind === 'error' ? 'alert' : 'status'}
  onmouseenter={() => (paused = true)}
  onmouseleave={() => (paused = false)}
  onfocusin={() => (paused = true)}
  onfocusout={() => (paused = false)}
>
  <Icon name={kind === 'info' ? 'info' : 'warning'} />
  <span class="text">{@render text()}</span>
  {#if kind === 'error'}
    <button type="button" class="dismiss" aria-label="Dismiss" onclick={() => ondismiss()}>
      <Icon name="close" />
    </button>
  {/if}
</div>

<style>
  .toast {
    display: flex;
    align-items: center;
    gap: var(--ss-space-3);
    padding: var(--ss-space-3) var(--ss-space-4);
    background: var(--ss-color-bg-raised);
    border: var(--ss-border-width) solid currentColor;
    border-radius: var(--ss-radius-sm);
    box-shadow: var(--ss-shadow-popover);
    z-index: var(--ss-z-toast);
  }
  .info {
    color: var(--ss-color-status-info);
  }
  .warning {
    color: var(--ss-color-status-warning);
  }
  .error {
    color: var(--ss-color-status-error);
  }
  .text {
    color: var(--ss-color-text-primary);
  }
  .dismiss {
    display: grid;
    place-items: center;
    width: var(--ss-size-control-sm);
    height: var(--ss-size-control-sm);
    padding: 0;
    border: 0;
    background: none;
    color: var(--ss-color-text-secondary);
    cursor: pointer;
  }
  .dismiss:hover {
    color: var(--ss-color-text-primary);
  }
</style>
