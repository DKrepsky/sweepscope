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
  const interpolation = $derived([
    { id: 'sinc', label: 'sin(x)/x', disabled: true, disabledReason: 'sin(x)/x interpolation is not available yet' },
    { id: 'linear', label: 'Linear', disabled: off, disabledReason: reason },
    { id: 'dots', label: 'Dots', disabled: off, disabledReason: reason },
  ]);

  const chooseInterpolation = (id: string) => actions.setInterpolation(id === 'dots' ? 'dots' : 'linear');
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
    />
    <Badge kind="software" tooltip="Interpolation is computed in software">SW</Badge>
  </div>
  <div class="note ss-num" title="Slot: effective sample rate. Value is illustrative.">{horizontal.rateNote}</div>
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
  .note {
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-secondary);
  }
</style>
