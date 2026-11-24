<script lang="ts">
  import type { ChannelModel, ChannelNumber, MathModel } from '../../app/main-window';
  import Badge from '../../ui/badge/Badge.svelte';
  import { channelStyle } from './channel-style';

  interface Props {
    channels: ChannelModel[];
    cursorSource: ChannelNumber;
    /** Shown while the dock hides the Trigger group (h4), so the trigger state stays at a glance. */
    triggerSummary?: string;
    math?: MathModel;
    /** Persistence is drawing: say where its update rate is. */
    persistenceNote?: boolean;
  }

  const CLIP_TIP =
    'The signal reaches the converter limits; amplitude measurements are not valid. Increase volts per division.';
  const PERSISTENCE_NOTE = 'Persistence builds from successive frames; the update rate is in the status bar.';
  const { channels, cursorSource, triggerSummary, math, persistenceNote }: Props = $props();

  const summary = (channel: ChannelModel) =>
    `${channel.voltsPerDiv.value} ${channel.voltsPerDiv.unit} ${channel.coupling.toUpperCase()}`;
  const shownText = (channel: ChannelModel) => (channel.enabled ? summary(channel) : 'off');
  const clip = (channel: ChannelModel) => (channel.enabled && channel.clipped ? ' Clip' : '');
  const role = (channel: ChannelModel) => (channel.enabled && channel.role ? ` ${channel.role.toUpperCase()}` : '');
  const alias = (channel: ChannelModel) => (channel.enabled && channel.alias ? ' Alias' : '');
  /** WCAG 2.5.3: the name starts with the visible text. */
  const accessibleName = (channel: ChannelModel) =>
    `${channel.number} ${shownText(channel)}${role(channel)}${alias(channel)}${clip(channel)}, channel ${channel.number} scale`;
</script>

<div class="strip" data-region="scale-strip" role="group" aria-label="Channel scales">
  {#each channels as channel (channel.number)}
    <button
      type="button"
      class="chip focus-ring"
      class:off={!channel.enabled}
      style={channelStyle(channel.number)}
      aria-pressed={channel.selected}
      aria-disabled={channel.enabled ? undefined : 'true'}
      aria-label={accessibleName(channel)}
    >
      <b>{channel.number}</b>
      <span class="ss-num">{shownText(channel)}</span>
      {#if channel.enabled && channel.role}
        <span class="role" data-role={channel.role}>{channel.role.toUpperCase()}</span>
      {/if}
      {#if channel.enabled && channel.alias}
        <Badge kind="warning" tooltip={channel.alias.message}>Alias</Badge>
      {/if}
      {#if channel.enabled && channel.clipped}
        <Badge kind="error" tooltip={CLIP_TIP}>Clip</Badge>
      {/if}
    </button>
  {/each}
  {#if math?.chip}
    <span class="chip math" class:unavailable={math.row.status === 'unavailable'} title={math.chip} data-math-chip>
      <span class="ss-num text">{math.chip}</span>
      {#if math.row.status === 'mixed' && math.row.statusDetail}
        <Badge kind="warning" tooltip={math.row.statusDetail}>Mixed</Badge>
      {/if}
    </span>
  {/if}
  <span class="grow"></span>
  {#if persistenceNote}<span class="note" data-persistence-note>{PERSISTENCE_NOTE}</span>{/if}
  {#if triggerSummary}<span class="note" data-trigger-summary>{triggerSummary}</span>{/if}
  <span class="note">
    Cursor values follow <b class="ss-num" style={channelStyle(cursorSource)}>CH{cursorSource}</b>
  </span>
</div>

<style>
  .strip {
    display: flex;
    min-inline-size: 0;
    align-items: center;
    gap: var(--ss-space-3);
    padding: var(--ss-space-3) var(--ss-space-4);
    background: var(--ss-color-bg-app);
    border-top: var(--ss-border-width) solid var(--ss-color-border-subtle);
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: var(--ss-space-3);
    block-size: var(--ss-size-control-md);
    padding: 0 var(--ss-space-3);
    font-size: var(--ss-font-size-sm);
    color: var(--ss-color-text-primary);
    background: var(--cht);
    border: var(--ss-border-width) solid var(--chc);
    border-radius: var(--ss-radius-sm);
  }
  .chip b {
    padding: 0 var(--ss-space-2);
    font-family: var(--ss-font-mono);
    color: var(--ss-color-text-on-accent);
    background: var(--chc);
    border-radius: var(--ss-radius-sm);
  }
  .chip.off {
    color: var(--ss-color-text-disabled);
    background: transparent;
    border-color: var(--ss-color-border-subtle);
  }
  .chip.off b {
    color: var(--ss-color-text-disabled);
    background: transparent;
    border: var(--ss-border-width) solid var(--ss-color-border-subtle);
  }
  .role {
    padding: 0 var(--ss-space-2);
    font-family: var(--ss-font-mono);
    font-size: var(--ss-font-size-2xs);
    font-weight: var(--ss-font-weight-bold);
    border: var(--ss-border-width) solid currentColor;
    border-radius: var(--ss-radius-sm);
  }
  .chip.math {
    min-inline-size: 0;
    border-color: var(--ss-color-math);
  }
  .chip.math .text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .chip.math.unavailable {
    color: var(--ss-color-text-secondary);
    border-style: dashed;
  }
  .grow {
    flex: 1;
  }
  .note {
    flex: none;
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-secondary);
  }
  .note b {
    color: var(--chc);
  }
</style>
