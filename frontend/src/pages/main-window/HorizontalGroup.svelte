<script lang="ts">
  import type { HorizontalModel, MainWindowActions } from '../../app/main-window';
  import Badge from '../../ui/badge/Badge.svelte';
  import Segmented from '../../ui/segmented/Segmented.svelte';
  import DockGroup from './DockGroup.svelte';
  import ReasonStepper from './ReasonStepper.svelte';

  interface Props {
    horizontal: HorizontalModel;
    actions: MainWindowActions;
  }

  const { horizontal, actions }: Props = $props();

  const off = $derived(horizontal.availability?.enabled === false);
  const reason = $derived(horizontal.availability?.reason);
  const automaticNote = 'Automatic: sin(x)/x below 250 samples per division, linear above, until you choose a mode.';
  const tips = $derived({
    sinc: `Smooth reconstruction computed in software. Rings at very sharp edges.${horizontal.automaticInterpolation ? ` ${automaticNote}` : ''}`,
    linear: 'Straight lines between the received samples.',
    dots: 'Real samples only, drawn as dots.',
  });
  const interpolation = $derived([
    { id: 'sinc', label: 'sin(x)/x', disabled: off, disabledReason: reason },
    { id: 'linear', label: 'Linear', disabled: off, disabledReason: reason },
    { id: 'dots', label: 'Dots', disabled: off, disabledReason: reason },
  ]);

  const modes = { sinc: 'sinc', linear: 'linear', dots: 'dots' } as const;
  const chooseInterpolation = (id: string) => {
    if (id in modes) actions.setInterpolation(modes[id as keyof typeof modes]);
  };
</script>

<DockGroup title="Horizontal" region="horizontal">
  <div class="line">
    <span class="ss-caps label">Time/div</span>
    <ReasonStepper
      value={horizontal.timePerDiv.value}
      unit={horizontal.timePerDiv.unit}
      smaller={horizontal.timePerDiv.step.faster}
      larger={horizontal.timePerDiv.step.slower}
      {off}
      {reason}
      onstep={actions.stepTimePerDiv}
      onreset={actions.resetTimePerDiv}
      valueText="{horizontal.timePerDiv.value} {horizontal.timePerDiv.unit}"
      ariaLabel="Time per division"
    />
    {#if horizontal.unverified}
      <Badge kind="warning" tooltip={horizontal.unverified}>Unverified</Badge>
    {/if}
  </div>
  <div class="line">
    <span class="ss-caps label">Trig pos</span>
    <ReasonStepper
      value={horizontal.triggerPosition.value}
      unit={horizontal.triggerPosition.unit}
      {off}
      {reason}
      onstep={actions.stepTriggerPosition}
      valueText="{horizontal.triggerPosition.value} percent"
      ariaLabel="Trigger position in percent of the record"
    />
  </div>
  <div class="line">
    <span class="ss-caps label">Interp</span>
    <Segmented
      value={horizontal.interpolation}
      options={interpolation}
      onchange={chooseInterpolation}
      ariaLabel="Interpolation"
    >
      {#snippet option({ id })}
        <span title={tips[id as keyof typeof tips]}>{interpolation.find((item) => item.id === id)?.label}</span>
      {/snippet}
    </Segmented>
    <Badge
      kind="software"
      tooltip={horizontal.automaticInterpolation ? automaticNote : 'Interpolation is computed in software'}
    >
      SW
    </Badge>
  </div>
  <div class="note ss-num" title="Slot: effective sample rate. Value is illustrative.">{horizontal.rateNote}</div>
</DockGroup>

<style>
  .line {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--ss-space-1) var(--ss-space-3);
    min-block-size: var(--ss-size-control-md);
  }
  .label {
    flex: none;
    inline-size: 3.6rem;
  }
  .note {
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-secondary);
  }
</style>
