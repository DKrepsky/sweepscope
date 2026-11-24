<script lang="ts">
  import type { Snippet } from 'svelte';
  import Tooltip from '../tooltip/Tooltip.svelte';
  import { nextIndex } from './navigation';
  import type { MenuItem } from './types';

  interface Props {
    items: MenuItem[];
    onselect: (id: string) => void;
    open: boolean;
    onclose: () => void;
    anchor: { x: number; y: number };
    item?: Snippet<[{ id: string }]>;
  }

  const { items, onselect, open, onclose, anchor, item }: Props = $props();

  let buttons: HTMLButtonElement[] = $state([]);
  let menu: HTMLElement | undefined = $state();
  let position = $state<{ x: number; y: number } | null>(null);

  const fitted = (start: number, size: number, room: number) => Math.max(0, Math.min(start, room - size));

  $effect(() => {
    if (!open) {
      position = null;
      return;
    }
    if (!menu) return;
    void items;

    const { width, height } = menu.getBoundingClientRect();
    position = {
      x: fitted(anchor.x, width, window.innerWidth),
      y: fitted(anchor.y, height, window.innerHeight),
    };
  });

  $effect(() => {
    if (!open) return;
    const opener = document.activeElement as HTMLElement | null;
    buttons[0]?.focus();
    return () => opener?.focus();
  });

  function activate(entry: MenuItem) {
    if (entry.disabled) return;
    onselect(entry.id);
  }

  function onkeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      onclose();
      return;
    }
    if (event.key === 'Tab') {
      event.preventDefault();
      onclose();
      return;
    }
    const target = nextIndex(event.key, buttons.indexOf(document.activeElement as HTMLButtonElement), items.length);
    if (target === null) return;
    event.preventDefault();
    buttons[target]?.focus();
  }
</script>

{#snippet row(entry: MenuItem, index: number)}
  <button
    bind:this={buttons[index]}
    type="button"
    role={entry.checked === undefined ? 'menuitem' : 'menuitemcheckbox'}
    class="entry"
    aria-checked={entry.checked === undefined ? undefined : entry.checked}
    aria-disabled={entry.disabled ? 'true' : undefined}
    tabindex={index === 0 ? 0 : -1}
    onclick={() => activate(entry)}
  >
    {#if entry.checked !== undefined}<span class="check" aria-hidden="true">{entry.checked ? '✓' : ''}</span>{/if}
    {#if item}{@render item({ id: entry.id })}{:else}<span class="label">{entry.label}</span>{/if}
    {#if entry.badge}<span class="badge">{entry.badge}</span>{/if}
  </button>
{/snippet}

{#if open}
  <div class="scrim" role="presentation" onpointerdown={onclose}></div>
  <div
    bind:this={menu}
    class="menu"
    role="menu"
    tabindex="-1"
    style:left="{(position ?? anchor).x}px"
    style:top="{(position ?? anchor).y}px"
    {onkeydown}
  >
    {#each items as entry, index (entry.id)}
      {#if entry.disabled && entry.disabledReason}
        <Tooltip text={entry.disabledReason}>{@render row(entry, index)}</Tooltip>
      {:else}
        {@render row(entry, index)}
      {/if}
    {/each}
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: var(--ss-z-popover);
  }
  .menu {
    position: fixed;
    z-index: var(--ss-z-popover);
    display: flex;
    flex-direction: column;
    min-width: 12rem;
    padding: var(--ss-space-2) 0;
    background: var(--ss-color-bg-raised);
    border: var(--ss-border-width) solid var(--ss-color-border-strong);
    box-shadow: var(--ss-shadow-popover);
    border-radius: var(--ss-radius-sm);
  }
  .entry {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--ss-space-4);
    width: 100%;
    height: var(--ss-size-control-md);
    padding: 0 var(--ss-space-4);
    border: 0;
    background: none;
    color: var(--ss-color-text-primary);
    font-family: var(--ss-font-ui);
    font-size: var(--ss-font-size-sm);
    text-align: left;
    cursor: pointer;
  }
  .entry:hover,
  .entry:focus-visible {
    background: var(--ss-color-bg-hover);
  }
  .entry[aria-disabled='true'] {
    color: var(--ss-color-text-disabled);
    cursor: not-allowed;
  }
  .check {
    inline-size: 1em;
    margin-inline-end: calc(-1 * var(--ss-space-2));
  }
  .badge {
    font-size: var(--ss-font-size-2xs);
    letter-spacing: var(--ss-letter-spacing-caps);
    text-transform: uppercase;
    color: var(--ss-color-text-muted);
  }
</style>
