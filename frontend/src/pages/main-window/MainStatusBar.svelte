<script lang="ts">
  import type { StatusModel } from '../../app/main-window';
  import Button from '../../ui/button/Button.svelte';
  import StatusBar from '../../ui/status-bar/StatusBar.svelte';
  import StatusChip from '../../ui/status-chip/StatusChip.svelte';
  import AnchoredPopover from './AnchoredPopover.svelte';

  interface Props {
    status: StatusModel;
    /** W3 and below: everything after connection and sample rate may wrap. */
    wrap: boolean;
  }

  const { status, wrap }: Props = $props();

  let accuracyOpen = $state(false);

  /** Order is the prototype's; priorities let items below `Never wrapped` stay on the first row (design 3.2). */
  const items = $derived([
    { id: 'connection', priority: 0, content: connection },
    { id: 'sample-rate', priority: 1, content: sampleRate },
    { id: 'record', priority: 2, content: record },
    { id: 'capture', priority: 3, content: capture },
    { id: 'renderer', priority: 4, content: renderer },
    { id: 'accuracy', priority: 5, content: accuracy },
  ]);
</script>

{#snippet connection()}
  <span class="connection" title={status.connection.tooltip}>
    <StatusChip kind="ok">{status.connection.state}</StatusChip>
    <span class="k">{status.connection.detail}</span>
  </span>
{/snippet}

{#snippet sampleRate()}
  <span title={status.sampleRate.tooltip}>
    <span class="k">Fs</span> <span class="v ss-num">{status.sampleRate.value}</span>
  </span>
{/snippet}

{#snippet record()}
  <span>
    <span class="k">Record</span> <span class="v ss-num">{status.record.points}</span>
    <span class="k">{status.record.span}</span>
  </span>
{/snippet}

{#snippet capture()}
  <span title={status.capture.tooltip}>
    <span class="k">Capture</span> <span class="v ss-num">{status.capture.rate}</span>
    <span class="k">{status.capture.deadTime}</span>
  </span>
{/snippet}

{#snippet renderer()}
  <span><span class="k">Renderer</span> <span class="v ss-num">{status.renderer}</span></span>
{/snippet}

{#snippet accuracy()}
  <span class="anchor">
    <Button size="sm" pressed={accuracyOpen} title="Accuracy note" onclick={() => (accuracyOpen = !accuracyOpen)}>
      Accuracy {status.accuracy}
    </Button>
    {#if accuracyOpen}
      <AnchoredPopover label="Accuracy note" placement="above-end" onclose={() => (accuracyOpen = false)}>
        {#snippet title()}Accuracy{/snippet}
        <p class="popover-text">Accuracy note slot. The wording comes from PRD 5.2 and is a placeholder here.</p>
      </AnchoredPopover>
    {/if}
  </span>
{/snippet}

<div data-region="status">
  <StatusBar {items} wrapPriorityAtLeast={wrap ? 2 : undefined} />
</div>

<style>
  .connection {
    display: inline-flex;
    align-items: center;
    gap: var(--ss-space-3);
    color: var(--ss-color-text-primary);
  }
  .k {
    color: var(--ss-color-text-muted);
  }
  .v {
    color: var(--ss-color-text-primary);
    font-size: var(--ss-font-size-sm);
  }
  .anchor {
    position: relative;
    display: inline-flex;
  }
  .popover-text {
    max-inline-size: 18rem;
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-secondary);
  }
</style>
