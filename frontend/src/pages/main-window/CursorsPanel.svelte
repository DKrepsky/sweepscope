<script lang="ts">
  import type { CursorsModel } from '../../app/main-window';
  import Button from '../../ui/button/Button.svelte';
  import Panel from '../../ui/panel/Panel.svelte';
  import { channelStyle } from './channel-style';

  interface Props {
    cursors: CursorsModel;
    onclear: () => void;
  }

  const { cursors, onclear }: Props = $props();

  let collapsed = $state(false);
</script>

<Panel {collapsed} ontoggle={() => (collapsed = !collapsed)}>
  {#snippet header()}Cursors{/snippet}
  {#snippet actions()}
    <span class="note">on <b class="ss-num" style={channelStyle(cursors.source)}>CH{cursors.source}</b></span>
    <Button
      size="sm"
      title="Remove all cursors (double-click a cursor removes it)"
      disabled={!cursors.enabled}
      disabledReason={cursors.available.reason ?? 'No cursors'}
      onclick={onclear}
    >
      Clear
    </Button>
  {/snippet}
  {#snippet body()}
    <div class="kv ss-num">
      {#each cursors.values as entry (entry.label)}
        <span class="k">{entry.label}</span>
        <span class="v" class:strong={entry.emphasised}>{entry.value}</span>
      {/each}
    </div>
    <p class="footnote">{cursors.originNote}</p>
  {/snippet}
</Panel>

<style>
  .note {
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-secondary);
  }
  .note b {
    color: var(--chc);
  }
  .kv {
    display: grid;
    grid-template-columns: auto 1fr auto 1fr;
    gap: var(--ss-space-2) var(--ss-space-4);
    align-items: baseline;
    font-size: var(--ss-font-size-md);
  }
  .k {
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-secondary);
  }
  .v {
    text-align: end;
  }
  .strong {
    color: var(--ss-color-text-primary);
  }
  .footnote {
    margin-block-start: var(--ss-space-3);
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-secondary);
  }
</style>
