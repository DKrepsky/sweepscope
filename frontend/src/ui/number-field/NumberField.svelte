<script module lang="ts">
  import type { Snippet } from 'svelte';

  export interface NumberFieldProps {
    value: string;
    commit: (text: string) => void;
    error?: string;
    disabled?: boolean;
    disabledReason?: string;
    ariaLabel: string;
    left?: Snippet;
    unit?: Snippet;
  }
</script>

<script lang="ts">
  import Tooltip from '../tooltip/Tooltip.svelte';

  let { value, commit, error, disabled = false, disabledReason, ariaLabel, left, unit }: NumberFieldProps = $props();

  const messageId = $props.id();
  let draft = $state<string | null>(null);
  const shown = $derived(draft ?? value);
  const tip = $derived(disabled ? disabledReason : undefined);

  function onkeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      event.preventDefault();
      submit();
    } else if (event.key === 'Escape') {
      draft = null;
    }
  }

  function submit() {
    if (disabled || draft === null) return;
    const text = draft;
    draft = null;
    commit(text);
  }

  function oninput(event: Event & { currentTarget: HTMLInputElement }) {
    if (!disabled) draft = event.currentTarget.value;
  }
</script>

{#snippet control()}
  <span class={['nf', { invalid: error, disabled }]}>
    {@render left?.()}
    <input
      class="ss-num focus-ring"
      type="text"
      inputmode="decimal"
      autocomplete="off"
      spellcheck="false"
      aria-label={ariaLabel}
      aria-invalid={error ? 'true' : undefined}
      aria-describedby={messageId}
      aria-disabled={disabled ? 'true' : undefined}
      readonly={disabled}
      value={shown}
      {oninput}
      {onkeydown}
      onblur={() => (draft = null)}
    />
    {@render unit?.()}
  </span>
{/snippet}

<span class="field">
  {#if tip}
    <Tooltip text={tip}>{@render control()}</Tooltip>
  {:else}
    {@render control()}
  {/if}
  <span class="msg" id={messageId} aria-live="polite">{error ?? ''}</span>
</span>

<style>
  .field {
    position: relative;
    display: inline-flex;
  }

  .nf {
    display: inline-flex;
    align-items: center;
    justify-content: flex-end;
    gap: var(--ss-space-2);
    height: var(--ss-size-control-md);
    min-width: 4.5rem;
    padding: 0 var(--ss-space-2);
    background: var(--ss-color-bg-inset);
    border: var(--ss-border-width) solid var(--ss-color-border);
    border-radius: var(--ss-radius-sm);
    font-size: var(--ss-font-size-md);
  }

  .nf :global(small) {
    color: var(--ss-color-text-muted);
    font-size: var(--ss-font-size-xs);
  }

  .invalid {
    border-color: var(--ss-color-status-error);
  }

  .disabled {
    color: var(--ss-color-text-disabled);
    background: var(--ss-color-bg-panel);
    border-color: var(--ss-color-border-subtle);
    cursor: not-allowed;
  }

  input {
    width: 100%;
    min-width: 0;
    background: none;
    border: 0;
    color: inherit;
    font-family: var(--ss-font-mono);
    font-size: inherit;
    text-align: right;
  }

  input:focus-visible {
    outline: var(--ss-border-width-focus) solid var(--ss-color-focus);
    outline-offset: var(--ss-border-width);
  }

  .msg {
    position: absolute;
    top: 100%;
    left: 0;
    z-index: var(--ss-z-overlay);
    height: var(--ss-size-control-sm);
    color: var(--ss-color-status-error);
    font-size: var(--ss-font-size-xs);
    white-space: nowrap;
    pointer-events: none;
  }
</style>
