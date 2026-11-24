<script lang="ts">
  import type {
    ChannelModel,
    ChannelNumber,
    MainWindowActions,
    PulseCondition,
    TriggerModel,
  } from '../../app/main-window';
  import Badge from '../../ui/badge/Badge.svelte';
  import Icon from '../../ui/icon/Icon.svelte';
  import NumberField from '../../ui/number-field/NumberField.svelte';
  import Segmented from '../../ui/segmented/Segmented.svelte';
  import DockGroup from './DockGroup.svelte';
  import ReasonStepper from './ReasonStepper.svelte';

  interface Props {
    trigger: TriggerModel;
    channels: ChannelModel[];
    actions: MainWindowActions;
  }

  const { trigger, channels, actions }: Props = $props();

  const CONDITIONS: { id: PulseCondition; symbol: string; name: string }[] = [
    { id: 'greater', symbol: '>', name: 'Longer than' },
    { id: 'less', symbol: '<', name: 'Shorter than' },
    { id: 'equal', symbol: '=', name: 'Equal to' },
    { id: 'notEqual', symbol: '≠', name: 'Not equal to' },
  ];

  const off = $derived(trigger.availability?.enabled === false);
  const reason = $derived(trigger.availability?.reason);
  const pulse = $derived(trigger.type === 'pulse' ? trigger.pulse : undefined);
  const choice = (options: { id: string; label: string }[]) =>
    options.map((option) => ({ ...option, disabled: off, disabledReason: reason }));
  const types = $derived([
    { id: 'edge', label: 'Edge', disabled: off, disabledReason: reason },
    { id: 'pulse', label: 'Pulse', disabled: off, disabledReason: reason },
    {
      id: 'video',
      label: 'Video',
      disabled: true,
      disabledReason: 'Video trigger is untested on this hardware (experimental)',
    },
  ]);
  const polarityNames = $derived(
    pulse
      ? { rising: 'Positive pulse', falling: 'Negative pulse' }
      : { rising: 'Rising edge', falling: 'Falling edge' },
  );
  const slopes = $derived(
    choice([
      { id: 'rising', label: polarityNames.rising },
      { id: 'falling', label: polarityNames.falling },
    ]),
  );
  const conditions = $derived(choice(CONDITIONS.map(({ id, name }) => ({ id, label: name }))));
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
  const conditionOf = (id: string) => CONDITIONS.find((c) => c.id === id);
  const slopeIcon = (id: string) => {
    const rising = id === 'rising';
    if (pulse) return rising ? 'pulse-positive' : 'pulse-negative';
    return rising ? 'rising-edge' : 'falling-edge';
  };

  function escapeClears(event: KeyboardEvent) {
    if (event.key === 'Escape') actions.clearPulseWidthError();
  }
</script>

<DockGroup title="Trigger" region="trigger">
  <div class="line">
    <span class="ss-caps label">Type</span>
    <span title={pulse ? trigger.accuracyNote : undefined}>
      <Segmented
        value={trigger.type}
        options={types}
        onchange={(id) => actions.setTriggerKind(id === 'pulse' ? 'pulse' : 'edge')}
        ariaLabel="Trigger type"
      />
    </span>
    <Badge kind="experimental" tooltip="Experimental: video triggers are untested">Exp</Badge>
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
      ariaLabel={pulse ? 'Trigger polarity' : 'Trigger slope'}
    >
      {#snippet option({ id })}
        <Icon name={slopeIcon(id)} />
        <span class="hidden-label">{id === 'rising' ? polarityNames.rising : polarityNames.falling}</span>
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
  {#if pulse}
    <div class="line">
      <span class="ss-caps label">Width</span>
      <Segmented
        value={pulse.condition}
        options={conditions}
        onchange={(id) => actions.setPulseCondition(conditionOf(id)?.id ?? 'greater')}
        ariaLabel="Pulse width condition"
      >
        {#snippet option({ id })}
          <span aria-hidden="true" title={conditionOf(id)?.name}>{conditionOf(id)?.symbol}</span>
          <span class="hidden-label">{conditionOf(id)?.name}</span>
        {/snippet}
      </Segmented>
      <!-- Escape and leaving the field both drop the shown error; the inner input owns the keys. -->
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <span role="group" aria-label="Pulse width" onkeydown={escapeClears} onfocusout={actions.clearPulseWidthError}>
        <NumberField
          value={pulse.width.value}
          commit={actions.commitPulseWidth}
          disabled={off}
          disabledReason={reason}
          error={pulse.error}
          errorText="external"
          ariaLabel="Pulse width"
        >
          {#snippet left()}<small>WIDTH</small>{/snippet}
          {#snippet unit()}<small>{pulse.width.unit}</small>{/snippet}
        </NumberField>
      </span>
    </div>
  {/if}
  <div class="line">
    <span class="ss-caps label">Sweep</span>
    <Segmented
      value={trigger.sweep}
      options={sweeps}
      onchange={(id) => actions.setTriggerSweep(id === 'normal' ? 'normal' : 'auto')}
      ariaLabel="Trigger sweep"
    />
  </div>
  {#if off && reason}<div class="reason" data-testid="trigger-reason">{reason}</div>{/if}
  {#if pulse}
    <div class="slot" role="status">{pulse.error ?? ''}</div>
  {:else if trigger.accuracyNote}
    <div class="note">{trigger.accuracyNote}</div>
  {/if}
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
  .reason {
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-status-warning);
  }
  /* Two lines are always reserved so a rejected width never moves the group. */
  .slot {
    min-block-size: 2lh;
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-status-error);
  }
  .note {
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-secondary);
  }
</style>
