<script module lang="ts">
  export type StepChipState = 'done' | 'current' | 'todo';

  export interface StepChipProps {
    label: string;
    state: StepChipState;
    /** CSS colour value of the step, normally a token variable. */
    color: string;
    /** Accessible name, for example `Channel 2, current`; defaults to the label. */
    name?: string;
    /** A done chip shows a check before its label. */
    check?: boolean;
    compact?: boolean;
  }
</script>

<script lang="ts">
  import Icon from '../icon/Icon.svelte';

  const { label, state, color, name, check = true, compact = false }: StepChipProps = $props();
</script>

<span
  class="chip {state}"
  class:compact
  style:--chc={color}
  aria-label={name ?? label}
  aria-current={state === 'current' ? 'step' : undefined}
>
  {#if state === 'done' && check}<Icon name="ok" />{/if}
  {label}
</span>

<style>
  .chip {
    display: inline-flex;
    align-items: center;
    gap: var(--ss-space-2);
    box-sizing: border-box;
    block-size: var(--ss-size-control-md);
    padding-inline: var(--ss-space-3);
    border: var(--ss-border-width) solid var(--chc);
    border-radius: var(--ss-radius-sm);
    font-family: var(--ss-font-mono);
    font-weight: var(--ss-font-weight-bold);
    color: var(--chc);
  }
  .compact {
    block-size: 1.25rem;
  }
  .done {
    background: var(--chc);
    color: var(--ss-color-text-on-accent);
  }
  .current {
    box-shadow:
      0 0 0 var(--ss-border-width-focus) var(--ss-color-bg-raised),
      0 0 0 calc(var(--ss-border-width-focus) + var(--ss-border-width)) var(--chc);
  }
  .todo {
    border-color: var(--ss-color-border);
    color: var(--ss-color-text-muted);
  }
  .chip :global(svg) {
    inline-size: 0.75rem;
    block-size: 0.75rem;
  }
</style>
