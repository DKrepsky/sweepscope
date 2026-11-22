<script lang="ts">
  import type { ChannelModel, TriggerModel } from '../../app/main-window';
  import Badge from '../../ui/badge/Badge.svelte';
  import Icon from '../../ui/icon/Icon.svelte';
  import Segmented from '../../ui/segmented/Segmented.svelte';
  import Stepper from '../../ui/stepper/Stepper.svelte';
  import DockGroup from './DockGroup.svelte';

  interface Props {
    trigger: TriggerModel;
    channels: ChannelModel[];
  }

  const { trigger, channels }: Props = $props();

  const noop = () => {};
  const types = [
    { id: 'edge', label: 'Edge' },
    { id: 'pulse', label: 'Pulse' },
    { id: 'video', label: 'Video', disabled: true, disabledReason: 'Video trigger is untested on this hardware' },
  ];
  const slopes = [
    { id: 'rising', label: 'Rising' },
    { id: 'falling', label: 'Falling' },
  ];
  const sweeps = [
    { id: 'auto', label: 'Auto' },
    { id: 'normal', label: 'Normal' },
  ];
  const sources = $derived(
    channels.map((channel) => ({
      id: String(channel.number),
      label: String(channel.number),
      disabled: !channel.enabled,
      disabledReason: `Channel ${channel.number} is off`,
    })),
  );
</script>

<DockGroup title="Trigger" region="trigger">
  <div class="line">
    <span class="ss-caps label">Type</span>
    <Segmented value={trigger.type} options={types} onchange={noop} ariaLabel="Trigger type" />
    <Badge kind="experimental" tooltip="Experimental: pulse and video triggers are not verified on this hardware">
      Exp
    </Badge>
  </div>
  <div class="line">
    <span class="ss-caps label">Source</span>
    <Segmented value={String(trigger.source)} options={sources} onchange={noop} ariaLabel="Trigger source">
      {#snippet option({ id })}<span style:color="var(--ss-color-ch{id})">{id}</span>{/snippet}
    </Segmented>
    <Segmented value={trigger.slope} options={slopes} onchange={noop} ariaLabel="Trigger slope">
      {#snippet option({ id })}
        <Icon name={id === 'rising' ? 'rising-edge' : 'falling-edge'} />
        <span class="hidden-label">{id === 'rising' ? 'Rising' : 'Falling'}</span>
      {/snippet}
    </Segmented>
  </div>
  <div class="line">
    <span class="ss-caps label">Level</span>
    <Stepper
      value={trigger.level.value}
      unit={trigger.level.unit}
      onstep={noop}
      valueText="{trigger.level.value} {trigger.level.unit}"
      ariaLabel="Trigger level"
    />
  </div>
  <div class="line">
    <span class="ss-caps label">Sweep</span>
    <Segmented value={trigger.sweep} options={sweeps} onchange={noop} ariaLabel="Trigger sweep" />
  </div>
  <div class="note" title="Trigger position accuracy statement, wording TBD">{trigger.accuracyNote}</div>
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
