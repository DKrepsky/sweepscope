<script lang="ts">
  import type { CalibrationTagModel } from '../../app/calibration';
  import type { StatusModel } from '../../app/main-window';
  import ConnectionItem, { type ConnectionItemState } from '../../ui/connection-item/ConnectionItem.svelte';
  import Icon from '../../ui/icon/Icon.svelte';
  import StatusBar from '../../ui/status-bar/StatusBar.svelte';
  import CalibrationTag from './CalibrationTag.svelte';

  const CONNECTION_STATES: Record<StatusModel['connection']['state'], ConnectionItemState> = {
    Ready: 'ready',
    Connecting: 'connecting',
    Disconnected: 'disconnected',
    Error: 'error',
  };

  interface Props {
    status: StatusModel;
    calibration: CalibrationTagModel;
    onopencalibration: () => void;
    /** W3 and below: everything after connection and sample rate may wrap. */
    wrap: boolean;
    /** h3 and below: only connection and sample rate stay (they never drop, design 3.2). */
    essentialOnly: boolean;
  }

  const { status, calibration, onopencalibration, wrap, essentialOnly }: Props = $props();

  /** The tag shares the priority of the sample rate: it never wraps and survives the collapse (design 2). */
  const ESSENTIAL_PRIORITY = 1;
  const allItems = $derived([
    { id: 'connection', priority: 0, content: connection },
    { id: 'sample-rate', priority: 1, content: sampleRate },
    { id: 'record', priority: 2, content: record },
    { id: 'capture', priority: 3, content: capture },
    { id: 'renderer', priority: 4, content: renderer },
    { id: 'calibration', priority: ESSENTIAL_PRIORITY, end: true, content: calibrationTag },
  ]);
  const items = $derived(allItems.filter((item) => !essentialOnly || item.priority <= ESSENTIAL_PRIORITY));
</script>

{#snippet connection()}
  <ConnectionItem
    state={CONNECTION_STATES[status.connection.state]}
    word={status.connection.state}
    detail={status.connection.detail}
    title={status.connection.tooltip}
  />
{/snippet}

{#snippet sampleRate()}
  <span class:alias={status.sampleRate.alias} title={status.sampleRate.alias?.message ?? status.sampleRate.tooltip}>
    <span class="k">Fs</span> <span class="v ss-num">{status.sampleRate.value}</span>
    {#if status.sampleRate.alias}
      <span class="alias-note" role="status"><Icon name="warning" /> {status.sampleRate.alias.short}</span>
    {/if}
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

{#snippet calibrationTag()}
  <CalibrationTag tag={calibration} onopen={onopencalibration} />
{/snippet}

<div data-region="status">
  <StatusBar {items} wrapPriorityAtLeast={wrap ? 2 : undefined} />
</div>

<style>
  .k {
    color: var(--ss-color-text-muted);
  }
  .v {
    color: var(--ss-color-text-primary);
    font-size: var(--ss-font-size-sm);
  }
  .alias .k,
  .alias .v,
  .alias-note {
    color: var(--ss-color-status-warning);
  }
  .alias-note {
    display: inline-flex;
    align-items: center;
    gap: var(--ss-space-1);
    font-size: var(--ss-font-size-sm);
  }
</style>
