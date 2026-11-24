<script lang="ts">
  import type { ChannelModel, ChannelNumber, MainWindowActions, TriggerModel } from '../../app/main-window';
  import Badge from '../../ui/badge/Badge.svelte';
  import Icon from '../../ui/icon/Icon.svelte';
  import Segmented from '../../ui/segmented/Segmented.svelte';
  import DockGroup from './DockGroup.svelte';
  import ReasonStepper from './ReasonStepper.svelte';

  interface Props {
    trigger: TriggerModel;
    channels: ChannelModel[];
    actions: MainWindowActions;
  }

  const { trigger, channels, actions }: Props = $props();

  const off = $derived(trigger.availability?.enabled === false);
  const reason = $derived(trigger.availability?.reason);
  const choice = (options: { id: string; label: string }[]) =>
    options.map((option) => ({ ...option, disabled: off, disabledReason: reason }));
  const types = [
    { id: 'edge', label: 'Edge' },
    { id: 'pulse', label: 'Pulse', disabled: true, disabledReason: 'Pulse trigger arrives in a later release' },
    { id: 'video', label: 'Video', disabled: true, disabledReason: 'Video trigger is untested on this hardware' },
  ];
  const slopes = $derived(
    choice([
      { id: 'rising', label: 'Rising' },
      { id: 'falling', label: 'Falling' },
    ]),
  );
  const sweeps = $derived(
    choice([
      { id: 'auto', label: 'Auto' },
      { id: 'normal', label: 'Normal' },
    ]),
  );
  const sources = $derived(
    channels.map((channel) => ({
      id: String(channel.number),
      label: String(channel.number),
      disabled: off || !channel.enabled,
      disabledReason: off ? reason : `Channel ${channel.number} is off`,
    })),
  );
  const asChannel = (id: string): ChannelNumber => Number(id) as ChannelNumber;
</script>

<DockGroup title="Trigger" region="trigger">
  <div class="line">
    <span class="ss-caps label">Type</span>
    <Segmented value={trigger.type} options={types} onchange={() => {}} ariaLabel="Trigger type" />
    <Badge kind="experimental" tooltip="Experimental: pulse and video triggers are not verified on this hardware">
      Exp
    </Badge>
  </div>
  <div class="line">
    <span class="ss-caps label">Source</span>
    <Segmented
      value={String(trigger.source)}
      options={sources}
      onchange={(id) => actions.setTriggerSource(asChannel(id))}
      ariaLabel="Trigger source"
    >
      {#snippet option({ id })}<span style:color="var(--ss-color-ch{id})">{id}</span>{/snippet}
    </Segmented>
    <Segmented
      value={trigger.slope}
      options={slopes}
      onchange={(id) => actions.setTriggerSlope(id === 'falling' ? 'falling' : 'rising')}
      ariaLabel="Trigger slope"
    >
      {#snippet option({ id })}
        <Icon name={id === 'rising' ? 'rising-edge' : 'falling-edge'} />
        <span class="hidden-label">{id === 'rising' ? 'Rising' : 'Falling'}</span>
      {/snippet}
    </Segmented>
  </div>
  <div class="line">
    <span class="ss-caps label">Level</span>
    <ReasonStepper
      value={trigger.level.value}
      unit={trigger.level.unit}
      {off}
      {reason}
      onstep={actions.stepTriggerLevel}
      onreset={actions.resetTriggerLevel}
      valueText="{trigger.level.value} {trigger.level.unit}"
      ariaLabel="Trigger level"
    />
  </div>
  <div class="line">
    <span class="ss-caps label">Sweep</span>
    <Segmented
      value={trigger.sweep}
      options={sweeps}
      onchange={(id) => actions.setTriggerSweep(id === 'normal' ? 'normal' : 'auto')}
      ariaLabel="Trigger sweep"
    />
  </div>
  {#if trigger.accuracyNote}<div class="note">{trigger.accuracyNote}</div>{/if}
</DockGroup>

<style>
  .line {
    display: flex;
    align-items: center;
    gap: var(--ss-space-3);
    min-block-size: var(--ss-size-control-md);
  }
  .label {
    flex: none;
    inline-size: 3.6rem;
  }
  .hidden-label {
    position: absolute;
    inline-size: var(--ss-border-width);
    block-size: var(--ss-border-width);
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }
  .note {
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-secondary);
  }
</style>
