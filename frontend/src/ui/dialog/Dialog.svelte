<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    open: boolean;
    modal: boolean;
    onclose: () => void;
    title: Snippet;
    body: Snippet;
    actions?: Snippet;
  }

  const { open, modal, onclose, title, body, actions }: Props = $props();
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

<dialog bind:this={element} aria-labelledby={titleId} oncancel={requestClose}>
  <header id={titleId}>{@render title()}</header>
  <div class="body">{@render body()}</div>
  {#if actions}<footer>{@render actions()}</footer>{/if}
</dialog>

<style>
  dialog {
    padding: 0;
    max-width: 28rem;
    color: var(--ss-color-text-primary);
    background: var(--ss-color-bg-raised);
    border: var(--ss-border-width) solid var(--ss-color-border-strong);
    box-shadow: var(--ss-shadow-dialog);
    border-radius: var(--ss-radius-md);
    z-index: var(--ss-z-dialog);
  }
  dialog::backdrop {
    background: var(--ss-color-bg-overlay);
  }
  header {
    padding: var(--ss-space-4) var(--ss-space-5);
    border-bottom: var(--ss-border-width) solid var(--ss-color-border);
    font-weight: var(--ss-font-weight-bold);
  }
  .body {
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
