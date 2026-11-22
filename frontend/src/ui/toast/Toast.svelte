<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from '../icon/Icon.svelte';

  interface Props {
    kind: 'info' | 'warning' | 'error';
    text: Snippet;
    timeoutMs?: number;
    ondismiss: () => void;
  }

  const { kind, text, timeoutMs = 4000, ondismiss }: Props = $props();

  $effect(() => {
    const timer = setTimeout(ondismiss, timeoutMs);
    return () => clearTimeout(timer);
  });
</script>

<div class="toast {kind}" role={kind === 'error' ? 'alert' : 'status'}>
  <Icon name={kind === 'info' ? 'info' : 'warning'} />
  <span class="text">{@render text()}</span>
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
</style>
