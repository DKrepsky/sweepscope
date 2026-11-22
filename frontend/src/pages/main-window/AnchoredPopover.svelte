<script lang="ts">
  import type { Snippet } from 'svelte';
  import Panel from '../../ui/panel/Panel.svelte';

  interface Props {
    label: string;
    placement: 'below-end' | 'above-end';
    onclose: () => void;
    title: Snippet;
    children: Snippet;
  }

  const { label, placement, onclose, title, children }: Props = $props();

  let popover: HTMLElement;

  function onkeydown(event: KeyboardEvent) {
    if (event.key !== 'Escape') return;
    event.stopPropagation();
    onclose();
  }

  $effect(() => {
    const opener = document.activeElement;
    popover.focus();
    return () => {
      const focusLost = document.activeElement === document.body || popover.contains(document.activeElement);
      if (focusLost && opener instanceof HTMLElement) opener.focus();
    };
  });
</script>

<svelte:window {onkeydown} />

<div class="popover {placement}" role="dialog" aria-label={label} tabindex="-1" bind:this={popover}>
  <Panel header={title} body={children} ontoggle={onclose} />
</div>

<style>
  .popover {
    position: absolute;
    inset-inline-end: 0;
    z-index: var(--ss-z-popover);
    min-inline-size: 16rem;
    background: var(--ss-color-bg-raised);
    border: var(--ss-border-width) solid var(--ss-color-border-strong);
    box-shadow: var(--ss-shadow-popover);
    border-radius: var(--ss-radius-sm);
  }
  .below-end {
    inset-block-start: 100%;
  }
  .above-end {
    inset-block-end: 100%;
  }
</style>
