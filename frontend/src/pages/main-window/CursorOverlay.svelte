<script lang="ts">
  import type { CursorId, CursorsModel } from '../../app/main-window';
  import Cursor from '../../ui/cursor/Cursor.svelte';

  interface Props {
    cursors: CursorsModel;
    onmove: (id: CursorId, position: number) => void;
    onremove: (id: CursorId) => void;
  }

  const { cursors, onmove, onremove }: Props = $props();

  const percent = (fraction: number) => `${fraction * 100}%`;
</script>

{#if cursors.enabled}
  <div class="overlay">
    {#each cursors.lines as line (line.id)}
      <Cursor
        axis={line.axis}
        position={line.position}
        label="Cursor {line.id.toUpperCase()}"
        valueText={line.valueText}
        disabled={!cursors.available.enabled}
        disabledReason={cursors.available.reason}
        onmove={(position) => onmove(line.id, position)}
        onremove={() => onremove(line.id)}
      >
        {#snippet tag()}<span class="ss-num">{line.tag}</span>{/snippet}
      </Cursor>
    {/each}
    {#if cursors.delta}
      <span class="delta ss-num" aria-hidden="true" style:left={percent(cursors.delta.position)}>
        {cursors.delta.tag}
      </span>
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
  .delta {
    position: absolute;
    inset-block-start: var(--ss-space-7);
    z-index: var(--ss-z-markers);
    padding: var(--ss-space-1) var(--ss-space-2);
    font-size: var(--ss-font-size-xs);
    line-height: 1;
    background: var(--ss-color-bg-scope);
    border: var(--ss-border-width) solid currentcolor;
    border-radius: var(--ss-radius-sm);
    transform: translateX(-50%);
  }
</style>
