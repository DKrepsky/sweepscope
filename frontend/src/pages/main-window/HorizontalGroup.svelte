<script lang="ts">
  import type { HorizontalModel } from '../../app/main-window';
  import Badge from '../../ui/badge/Badge.svelte';
  import Segmented from '../../ui/segmented/Segmented.svelte';
  import Stepper from '../../ui/stepper/Stepper.svelte';
  import DockGroup from './DockGroup.svelte';

  interface Props {
    horizontal: HorizontalModel;
  }

  const { horizontal }: Props = $props();

  const noop = () => {};
  const interpolation = [
    { id: 'sinc', label: 'sin(x)/x' },
    { id: 'linear', label: 'Linear' },
    { id: 'dots', label: 'Dots' },
  ];
</script>

<DockGroup title="Horizontal" region="horizontal">
  <div class="line">
    <span class="ss-caps label">Time/div</span>
    <Stepper
      value={horizontal.timePerDiv.value}
      unit={horizontal.timePerDiv.unit}
      onstep={noop}
      valueText="{horizontal.timePerDiv.value} {horizontal.timePerDiv.unit}"
      ariaLabel="Time per division"
    />
  </div>
  <div class="line">
    <span class="ss-caps label">Trig pos</span>
    <Stepper
      value={horizontal.triggerPosition.value}
      unit={horizontal.triggerPosition.unit}
      onstep={noop}
      valueText="{horizontal.triggerPosition.value} percent"
      ariaLabel="Trigger position in percent of the record"
    />
  </div>
  <div class="line">
    <span class="ss-caps label">Interp</span>
    <Segmented value={horizontal.interpolation} options={interpolation} onchange={noop} ariaLabel="Interpolation" />
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
