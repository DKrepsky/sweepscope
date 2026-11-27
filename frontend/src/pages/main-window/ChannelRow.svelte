<script lang="ts">
  import type { ChannelModel, MainWindowActions } from '../../app/main-window';
  import Button from '../../ui/button/Button.svelte';
  import NumberField from '../../ui/number-field/NumberField.svelte';
  import Segmented from '../../ui/segmented/Segmented.svelte';
  import { channelStyle } from './channel-style';
  import ReasonStepper from './ReasonStepper.svelte';

  interface Props {
    channel: ChannelModel;
    actions: MainWindowActions;
  }

  const { channel, actions }: Props = $props();

  const CONTROLS = 'button, input, [role="spinbutton"]';
  const n = $derived(channel.number);
  const blocked = $derived(channel.availability?.enabled === false);
  const off = $derived(!channel.enabled || blocked);
  const reason = $derived(
    blocked ? channel.availability?.reason : !channel.enabled ? `Channel ${channel.number} is off` : undefined,
  );
  const toggleReason = $derived(blocked ? channel.availability?.reason : channel.enableReason);
  const name = $derived(`Channel ${channel.number}`);

  function select(event: MouseEvent) {
    if (!(event.target as HTMLElement).closest(CONTROLS)) actions.selectChannel(n);
  }

  function toggle() {
    if (!toggleReason) actions.setChannelEnabled(n, !channel.enabled);
  }
  const coupling = $derived(
    [
      { id: 'ac', label: 'AC' },
      { id: 'dc', label: 'DC' },
    ].map((option) => ({ ...option, disabled: off, disabledReason: reason })),
  );
</script>

<!-- Clicking the row's empty space selects the channel; every control inside has its own action. -->
<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div
  class="row"
  style={channelStyle(channel.number)}
  data-channel={channel.number}
  data-enabled={channel.enabled}
  data-selected={channel.selected}
  onclick={select}
>
  <button
    type="button"
    class="toggle focus-ring"
    class:outline={off}
    aria-pressed={channel.enabled}
    aria-disabled={toggleReason ? 'true' : undefined}
    title={toggleReason ?? `${name} on (Shift+${channel.number} toggles)`}
    aria-label="{name} enabled"
    onclick={toggle}
  >
    {channel.number}
  </button>
  <ReasonStepper
    value={channel.voltsPerDiv.value}
    unit={channel.voltsPerDiv.unit}
    smaller={channel.voltsPerDiv.step.faster}
    larger={channel.voltsPerDiv.step.slower}
    onstep={(direction) => actions.stepVoltsPerDiv(n, direction)}
    onreset={() => actions.resetVoltsPerDiv(n)}
    {off}
    {reason}
    valueText="{channel.voltsPerDiv.value} {channel.voltsPerDiv.unit}"
    ariaLabel="{name} volts per division"
  />
  <NumberField
    value={channel.position.value}
    commit={(text) => actions.commitChannelPosition(n, text)}
    disabled={off}
    disabledReason={reason}
    ariaLabel="{name} vertical position in divisions"
  >
    {#snippet left()}<small>POS</small>{/snippet}
    {#snippet unit()}<small>{channel.position.unit}</small>{/snippet}
  </NumberField>
  <span class="coupling" class:dim={off}>
    <Segmented
      value={channel.coupling}
      options={coupling}
      onchange={(id) => actions.setCoupling(n, id === 'ac' ? 'ac' : 'dc')}
      ariaLabel="{name} coupling"
    />
  </span>
  <Button
    pressed={channel.bandwidthLimit}
    disabled={off}
    disabledReason={reason}
    title="20 MHz bandwidth limit"
    ariaLabel="BW, {name} bandwidth limit"
    onclick={() => actions.toggleBandwidth(n)}
  >
    BW
  </Button>
  <Button
    disabled={off}
    disabledReason={reason}
    title="Probe attenuation"
    ariaLabel="{channel.probe}, {name} probe attenuation"
    onclick={() => actions.cycleProbe(n)}
  >
    {channel.probe}
  </Button>
  <Button
    pressed={channel.inverted}
    disabled={off}
    disabledReason={reason}
    title="Invert"
    ariaLabel="Inv, {name} invert"
    onclick={() => actions.toggleInvert(n)}
  >
    Inv
  </Button>
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
</style>
