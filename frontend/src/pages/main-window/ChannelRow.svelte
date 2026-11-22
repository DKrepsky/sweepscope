<script lang="ts">
  import type { ChannelModel } from '../../app/main-window';
  import Badge from '../../ui/badge/Badge.svelte';
  import Button from '../../ui/button/Button.svelte';
  import NumberField from '../../ui/number-field/NumberField.svelte';
  import Segmented from '../../ui/segmented/Segmented.svelte';
  import Stepper from '../../ui/stepper/Stepper.svelte';
  import { channelStyle } from './channel-style';

  interface Props {
    channel: ChannelModel;
  }

  const { channel }: Props = $props();

  const noop = () => {};
  const off = $derived(!channel.enabled);
  const reason = $derived(off ? `Channel ${channel.number} is off` : undefined);
  const name = $derived(`Channel ${channel.number}`);
  const coupling = $derived(
    [
      { id: 'ac', label: 'AC' },
      { id: 'dc', label: 'DC' },
    ].map((option) => ({ ...option, disabled: off, disabledReason: reason })),
  );
</script>

<div
  class="row"
  style={channelStyle(channel.number)}
  data-channel={channel.number}
  data-enabled={channel.enabled}
  data-selected={channel.selected}
>
  <button
    type="button"
    class="toggle focus-ring"
    class:outline={off}
    aria-pressed={channel.enabled}
    title={channel.enableReason ?? `${name} on (Shift+${channel.number} toggles)`}
    aria-label="{name} enabled"
  >
    {channel.number}
  </button>
  <Stepper
    value={channel.voltsPerDiv.value}
    unit={channel.voltsPerDiv.unit}
    onstep={noop}
    {off}
    valueText="{channel.voltsPerDiv.value} {channel.voltsPerDiv.unit}"
    ariaLabel="{name} volts per division"
  />
  <NumberField
    value={channel.position.value}
    commit={noop}
    disabled={off}
    disabledReason={reason}
    ariaLabel="{name} vertical position in divisions"
  >
    {#snippet left()}<small>POS</small>{/snippet}
    {#snippet unit()}<small>{channel.position.unit}</small>{/snippet}
  </NumberField>
  <span class="coupling" class:dim={off}>
    <Segmented value={channel.coupling} options={coupling} onchange={noop} ariaLabel="{name} coupling" />
  </span>
  <Button
    pressed={channel.bandwidthLimit}
    disabled={off}
    disabledReason={reason}
    title="20 MHz bandwidth limit"
    ariaLabel="BW, {name} bandwidth limit"
  >
    BW
  </Button>
  <Button
    disabled={off}
    disabledReason={reason}
    title="Probe attenuation"
    ariaLabel="{channel.probe}, {name} probe attenuation"
  >
    {channel.probe}
  </Button>
  <Button
    pressed={channel.inverted}
    disabled={off}
    disabledReason={reason}
    title="Invert"
    ariaLabel="Inv, {name} invert"
  >
    Inv
  </Button>
  {#if channel.unverified}
    <span class="badge">
      <Badge kind="warning" tooltip={channel.unverified}>?</Badge>
    </span>
  {/if}
</div>

<style>
  /* The columns come from the parent grid (VerticalGroup), so every row shares them. */
  .row {
    display: grid;
    grid-template-columns: subgrid;
    grid-column: 1 / -1;
    align-items: center;
    padding: var(--ss-space-1) var(--ss-space-2) var(--ss-space-1) 0;
    border-inline-start: calc(var(--ss-border-width) * 3) solid var(--chc);
  }
  .row[data-selected='true'] {
    background: var(--cht);
  }
  .toggle {
    display: grid;
    place-items: center;
    flex: none;
    inline-size: var(--ss-size-control-md);
    block-size: var(--ss-size-control-md);
    margin-inline-start: var(--ss-space-2);
    font-family: var(--ss-font-mono);
    font-weight: var(--ss-font-weight-bold);
    color: var(--ss-color-text-on-accent);
    background: var(--chc);
    border: var(--ss-border-width) solid var(--chc);
  }
  .toggle.outline {
    color: var(--chc);
    background: var(--ss-color-bg-inset);
  }
  .coupling {
    display: inline-flex;
  }
  /* Same dimming as the Stepper's off state, so a disabled row reads as one. */
  .coupling.dim {
    opacity: 0.45;
  }
  .badge {
    justify-self: end;
  }
</style>
