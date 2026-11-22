<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from '../icon/Icon.svelte';

  interface Props {
    collapsed?: boolean;
    header?: Snippet;
    body: Snippet;
    actions?: Snippet;
    ontoggle?: () => void;
  }

  const { collapsed = false, header, body, actions, ontoggle }: Props = $props();
</script>

<section class="panel" class:collapsed>
  <header>
    <button
      type="button"
      class="toggle"
      aria-expanded={!collapsed}
      aria-label="Toggle panel"
      onclick={() => ontoggle?.()}
    >
      <Icon name="chevron" />
    </button>
    <h2>{@render header?.()}</h2>
    <span class="grow"></span>
    {@render actions?.()}
  </header>
  {#if !collapsed}
    <div class="body">{@render body()}</div>
  {/if}
</section>

<style>
  .panel {
    border-bottom: var(--ss-border-width) solid var(--ss-color-border);
    padding: var(--ss-space-3) var(--ss-space-4) var(--ss-space-4);
  }
  .panel.collapsed {
    padding-bottom: var(--ss-space-3);
  }
  header {
    display: flex;
    align-items: center;
    gap: var(--ss-space-3);
  }
  .panel:not(.collapsed) header {
    margin-bottom: var(--ss-space-3);
  }
  h2 {
    font-size: var(--ss-font-size-2xs);
    letter-spacing: var(--ss-letter-spacing-caps);
    text-transform: uppercase;
    font-weight: var(--ss-font-weight-bold);
    color: var(--ss-color-text-secondary);
  }
  .grow {
    flex: 1;
  }
  .toggle {
    display: grid;
    place-items: center;
    width: var(--ss-size-control-sm);
    height: var(--ss-size-control-sm);
    padding: 0;
    border: 0;
    background: none;
    color: var(--ss-color-text-secondary);
    cursor: pointer;
    transition: transform var(--ss-motion-fast) var(--ss-motion-easing);
  }
  .toggle:hover {
    color: var(--ss-color-text-primary);
  }
  .toggle[aria-expanded='false'] {
    transform: rotate(-90deg);
  }
</style>
