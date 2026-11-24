<script lang="ts">
  import Icon from '../icon/Icon.svelte';
  import { splitRows } from './rows';
  import type { StatusBarItem } from './types';

  interface Props {
    items: StatusBarItem[];
    wrapPriorityAtLeast?: number;
  }

  const { items, wrapPriorityAtLeast }: Props = $props();

  const rows = $derived(splitRows(items, wrapPriorityAtLeast));
</script>

<footer class="status">
  {#each rows as row, index (index)}
    <div class="row">
      {#each row as item (item.id)}
        <div class="item" class:warning={item.warning} class:end={item.end} data-item={item.id}>
          {#if item.warning}<Icon name="warning" />{/if}
          {@render item.content()}
        </div>
      {/each}
    </div>
  {/each}
</footer>

<style>
  .status {
    display: flex;
    flex-direction: column;
    min-height: var(--ss-size-statusbar);
    background: var(--ss-color-bg-app);
    border-top: var(--ss-border-width) solid var(--ss-color-border);
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-secondary);
  }
  .row {
    display: flex;
    align-items: center;
    min-height: var(--ss-size-statusbar);
  }
  .row + .row {
    border-top: var(--ss-border-width) solid var(--ss-color-border-subtle);
  }
  .item {
    display: flex;
    align-items: center;
    gap: var(--ss-space-3);
    padding: 0 var(--ss-space-5);
    align-self: stretch;
    border-right: var(--ss-border-width) solid var(--ss-color-border-subtle);
  }
  .end {
    margin-inline-start: auto;
    border-right: 0;
  }
  .warning {
    color: var(--ss-color-status-warning);
  }
</style>
