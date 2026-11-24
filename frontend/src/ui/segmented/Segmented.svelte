<script module lang="ts">
  import type { Snippet } from 'svelte';

  export interface SegmentedOption {
    id: string;
    label?: string;
    disabled?: boolean;
    disabledReason?: string;
    badge?: string;
  }

  export interface SegmentedProps {
    value: string;
    options: SegmentedOption[];
    onchange: (id: string) => void;
    ariaLabel: string;
    option?: Snippet<[{ id: string; active: boolean }]>;
  }
</script>

<script lang="ts">
  import Tooltip from '../tooltip/Tooltip.svelte';

  let { value, options, onchange, ariaLabel, option }: SegmentedProps = $props();

  let buttons: HTMLButtonElement[] = $state([]);

  function select(item: SegmentedOption) {
    if (!item.disabled && item.id !== value) onchange(item.id);
  }

  function neighbour(from: number, step: 1 | -1): number | undefined {
    for (let i = from + step; i >= 0 && i < options.length; i += step) {
      if (!options[i]?.disabled) return i;
    }
    return undefined;
  }

  function navigate(event: KeyboardEvent, index: number) {
    const step = event.key === 'ArrowRight' ? 1 : event.key === 'ArrowLeft' ? -1 : 0;
    if (step === 0) return;
    event.preventDefault();
    const target = neighbour(index, step);
    if (target !== undefined) buttons[target]?.focus();
  }
</script>

<div class="seg" role="group" aria-label={ariaLabel}>
  {#each options as item, index (item.id)}
    {@const active = item.id === value}
    {#snippet control()}
      <button
        type="button"
        class={['focus-ring', { active, disabled: item.disabled }]}
        aria-pressed={active}
        aria-disabled={item.disabled ? 'true' : undefined}
        bind:this={buttons[index]}
        onclick={() => select(item)}
        onkeydown={(event) => navigate(event, index)}
      >
        {#if option}
          {@render option({ id: item.id, active })}
        {:else}
          {item.label ?? item.id}
        {/if}
        {#if item.badge}<span class="badge">{item.badge}</span>{/if}
      </button>
    {/snippet}
    {#if item.disabled && item.disabledReason}
      <Tooltip text={item.disabledReason}>{@render control()}</Tooltip>
    {:else}
      {@render control()}
    {/if}
  {/each}
</div>

<style>
  .seg {
    display: inline-flex;
    border: var(--ss-border-width) solid var(--ss-color-border-control);
    border-radius: var(--ss-radius-sm);
    background: var(--ss-color-bg-raised);
  }

  .seg > :global(* + *) {
    border-left: var(--ss-border-width) solid var(--ss-color-border-control);
  }

  button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--ss-space-2);
    height: calc(var(--ss-size-control-md) - 2 * var(--ss-border-width));
    min-width: var(--ss-size-control-md);
    padding: 0 var(--ss-space-3);
    font-size: var(--ss-font-size-sm);
    white-space: nowrap;
    color: var(--ss-color-text-secondary);
  }

  button:hover {
    background: var(--ss-color-bg-hover);
  }

  .focus-ring:focus-visible {
    outline: var(--ss-border-width-focus) solid var(--ss-color-focus);
    outline-offset: calc(-1 * var(--ss-border-width-focus));
  }

  .active {
    background: var(--ss-color-bg-selected);
    color: var(--ss-color-text-primary);
    box-shadow: inset 0 calc(-1 * var(--ss-border-width-focus)) 0 var(--ss-color-accent);
  }

  .disabled {
    background: var(--ss-color-bg-panel);
    color: var(--ss-color-text-disabled);
    cursor: not-allowed;
  }

  /* Muted text on the selected fill is 3.93:1; the secondary colour passes 4.5:1. */
  .active .badge {
    color: var(--ss-color-text-secondary);
  }

  .badge {
    font-size: var(--ss-font-size-2xs);
    color: var(--ss-color-text-muted);
    letter-spacing: var(--ss-letter-spacing-caps);
    text-transform: uppercase;
  }
</style>
