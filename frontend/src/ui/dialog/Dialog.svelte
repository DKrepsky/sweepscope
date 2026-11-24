<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    open: boolean;
    modal: boolean;
    onclose: () => void;
    title: Snippet;
    body: Snippet;
    actions?: Snippet;
    /** Next to the title, outside the dialog's accessible name. */
    meta?: Snippet;
    /** A full-width row between the header and the body. */
    strip?: Snippet;
    /** Id of the element inside the body that describes the dialog. */
    describedby?: string;
    wide?: boolean;
  }

  const { open, modal, onclose, title, body, actions, meta, strip, describedby, wide = false }: Props = $props();
  const titleId = $props.id();

  let element: HTMLDialogElement;

  $effect(() => {
    if (open && !element.open) openDialog();
    if (!open && element.open) element.close();
  });

  function openDialog() {
    if (modal) element.showModal();
    else element.show();
  }

  // The owner decides whether the dialog closes: the native Escape close is cancelled.
  function requestClose(event: Event) {
    event.preventDefault();
    onclose();
  }
</script>

<dialog bind:this={element} class:wide aria-labelledby={titleId} aria-describedby={describedby} oncancel={requestClose}>
  <header>
    <span id={titleId}>{@render title()}</span>
    {#if meta}<span class="meta">{@render meta()}</span>{/if}
  </header>
  {#if strip}{@render strip()}{/if}
  <div class="body">{@render body()}</div>
  {#if actions}<footer>{@render actions()}</footer>{/if}
</dialog>

<style>
  dialog {
    margin: auto;
    padding: 0;
    max-width: 28rem;
    color: var(--ss-color-text-primary);
    background: var(--ss-color-bg-raised);
    border: var(--ss-border-width) solid var(--ss-color-border-strong);
    box-shadow: var(--ss-shadow-dialog);
    border-radius: var(--ss-radius-md);
    z-index: var(--ss-z-dialog);
  }
  dialog[open] {
    display: flex;
    flex-direction: column;
  }
  dialog.wide {
    inline-size: min(34rem, 100% - 2rem);
    max-inline-size: none;
  }
  dialog::backdrop {
    background: var(--ss-color-scrim);
  }
  header {
    display: flex;
    align-items: center;
    gap: var(--ss-space-4);
    padding: var(--ss-space-4) var(--ss-space-5);
    border-bottom: var(--ss-border-width) solid var(--ss-color-border);
    font-weight: var(--ss-font-weight-bold);
  }
  .meta {
    margin-inline-start: auto;
    font-weight: var(--ss-font-weight-regular);
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-muted);
  }
  .body {
    min-block-size: 0;
    overflow-y: auto;
    display: grid;
    gap: var(--ss-space-4);
    padding: var(--ss-space-5);
    color: var(--ss-color-text-secondary);
  }
  footer {
    display: flex;
    gap: var(--ss-space-3);
    justify-content: flex-end;
    padding: var(--ss-space-4) var(--ss-space-5);
    border-top: var(--ss-border-width) solid var(--ss-color-border);
  }
</style>
