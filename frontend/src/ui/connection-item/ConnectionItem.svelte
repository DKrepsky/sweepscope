<script module lang="ts">
  export type ConnectionItemState = 'ready' | 'connecting' | 'disconnected' | 'error';

  export interface ConnectionItemProps {
    state: ConnectionItemState;
    /** The state word, shown bold; it carries the state, the dot colour is secondary. */
    word: string;
    detail: string;
    title?: string;
  }
</script>

<script lang="ts">
  import Icon from '../icon/Icon.svelte';

  const { state, word, detail, title }: ConnectionItemProps = $props();
</script>

<span class={['connection', state]} {title}>
  {#if state === 'error'}
    <Icon name="warning" />
  {:else}
    <span class="dot"></span>
  {/if}
  <b class="word">{word}</b>
  {#if detail}<span class="detail">{detail}</span>{/if}
</span>

<style>
  .connection {
    display: inline-flex;
    align-items: center;
    gap: var(--ss-space-3);
    color: var(--ss-color-status-ok);
  }
  .connecting {
    color: var(--ss-color-status-info);
  }
  .disconnected {
    color: var(--ss-color-status-disconnected);
  }
  .error {
    color: var(--ss-color-status-error);
  }
  .dot {
    inline-size: var(--ss-space-3);
    block-size: var(--ss-space-3);
    border-radius: 50%;
    background: currentColor;
  }
  .word {
    color: var(--ss-color-text-primary);
    font-weight: var(--ss-font-weight-bold);
  }
  .detail {
    color: var(--ss-color-text-muted);
  }
</style>
