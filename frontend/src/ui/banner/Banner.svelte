<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from '../icon/Icon.svelte';

  interface Props {
    kind: 'info' | 'warning' | 'error';
    ondismiss?: () => void;
    icon?: Snippet;
    text: Snippet;
    actions?: Snippet;
  }

  const { kind, ondismiss, icon, text, actions }: Props = $props();
</script>

<div class="banner {kind}" role={kind === 'error' ? 'alert' : 'status'}>
  {#if icon}{@render icon()}{:else}<Icon name={kind === 'info' ? 'info' : 'warning'} />{/if}
  <div class="text">{@render text()}</div>
  {#if actions}<div class="actions">{@render actions()}</div>{/if}
  {#if ondismiss}
    <button type="button" class="dismiss" aria-label="Dismiss" onclick={() => ondismiss()}>
      <Icon name="close" />
    </button>
  {/if}
</div>

<style>
  .banner {
    display: flex;
    align-items: center;
    gap: var(--ss-space-4);
    padding: var(--ss-space-3) var(--ss-space-4);
    border: var(--ss-border-width) solid currentColor;
    border-left-width: var(--ss-space-2);
    background: var(--ss-color-bg-raised);
    border-radius: var(--ss-radius-sm);
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
    flex: 1;
    color: var(--ss-color-text-primary);
  }
  .actions {
    display: flex;
    gap: var(--ss-space-3);
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
