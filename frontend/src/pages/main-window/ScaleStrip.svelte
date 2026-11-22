<script lang="ts">
  import type { ChannelModel, ChannelNumber } from '../../app/main-window';
  import Badge from '../../ui/badge/Badge.svelte';
  import { channelStyle } from './channel-style';

  interface Props {
    channels: ChannelModel[];
    cursorSource: ChannelNumber;
  }

  const { channels, cursorSource }: Props = $props();

  const summary = (channel: ChannelModel) =>
    `${channel.voltsPerDiv.value} ${channel.voltsPerDiv.unit} ${channel.coupling.toUpperCase()}`;
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
      aria-label="Channel {channel.number}: {channel.enabled ? summary(channel) : 'off'}"
    >
      <b>{channel.number}</b>
      <span class="ss-num">{channel.enabled ? summary(channel) : 'off'}</span>
      {#if channel.enabled && channel.unverified}
        <Badge kind="warning" tooltip={channel.unverified}>Unverified</Badge>
      {/if}
    </button>
  {/each}
  <span class="grow"></span>
  <span class="note">
    Cursor values follow <b class="ss-num" style={channelStyle(cursorSource)}>CH{cursorSource}</b>
  </span>
</div>

<style>
  .strip {
    display: flex;
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
  .grow {
    flex: 1;
  }
  .note {
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-secondary);
  }
  .note b {
    color: var(--chc);
  }
</style>
