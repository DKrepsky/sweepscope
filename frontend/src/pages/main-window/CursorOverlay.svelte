<script lang="ts">
  import type { CursorsModel } from '../../app/main-window';

  interface Props {
    cursors: CursorsModel;
  }

  const { cursors }: Props = $props();

  const percent = (fraction: number) => `${fraction * 100}%`;
</script>

{#if cursors.enabled}
  <div class="overlay" aria-hidden="true">
    {#each cursors.lines as line (line.id)}
      {#if line.axis === 'x'}
        <span class="line vertical" style:left={percent(line.position)}></span>
        <span class="tag ss-num bottom" style:left={percent(line.position)}>{line.tag}</span>
      {:else}
        <span class="line horizontal" style:top={percent(line.position)}></span>
        <span class="tag ss-num left" style:top={percent(line.position)}>{line.tag}</span>
      {/if}
    {/each}
    {#if cursors.delta}
      <span class="tag ss-num top" style:left={percent(cursors.delta.position)}>{cursors.delta.tag}</span>
    {/if}
  </div>
{/if}

<style>
  .overlay {
    position: absolute;
    inset: 0;
    pointer-events: none;
    color: var(--ss-color-cursor);
  }
  .line {
    position: absolute;
    border: 0 dashed currentColor;
  }
  .vertical {
    inset-block: 0;
    border-inline-start-width: var(--ss-border-width);
  }
  .horizontal {
    inset-inline: 0;
    border-block-start-width: var(--ss-border-width);
  }
  .tag {
    position: absolute;
    z-index: var(--ss-z-markers);
    padding: var(--ss-space-1) var(--ss-space-2);
    font-size: var(--ss-font-size-xs);
    line-height: 1;
    background: var(--ss-color-bg-overlay);
    border: var(--ss-border-width) solid currentColor;
    border-radius: var(--ss-radius-sm);
  }
  .bottom {
    inset-block-end: var(--ss-space-3);
    transform: translateX(-50%);
  }
  .top {
    inset-block-start: var(--ss-space-7);
    transform: translateX(-50%);
    background: var(--ss-color-bg-scope);
  }
  .left {
    inset-inline-start: var(--ss-space-3);
    transform: translateY(-50%);
  }
</style>
