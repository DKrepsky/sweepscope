<script lang="ts">
  import { iconNames } from '../../design-system';
  import { Badge } from '../../ui/badge';
  import { Button } from '../../ui/button';
  import { Icon } from '../../ui/icon';
  import { NumberField } from '../../ui/number-field';
  import { RunButton } from '../../ui/run-button';
  import { Segmented } from '../../ui/segmented';
  import { Stepper } from '../../ui/stepper';
  import Forced from './Forced.svelte';
  import Row from './Row.svelte';
  import Section from './Section.svelte';

  const noop = () => {};
  let slope = $state('auto');
  let triggerType = $state('edge');
  let running = $state(true);
</script>

<Section component="Icon">
  <Row>
    {#each iconNames as name (name)}
      <span class="icon-item" title={name}><Icon {name} /></span>
    {/each}
  </Row>
</Section>

<Section component="Button">
  <Row label="Button">
    <Button>Default</Button>
    <Forced state="hover"><Button>Hover</Button></Forced>
    <Forced state="pressed"><Button>Pressed</Button></Forced>
    <Forced state="focus"><Button>Focus</Button></Forced>
    <Button pressed>On</Button>
    <Button disabled disabledReason="Stop acquisition first">Disabled</Button>
  </Row>
  <Row label="Sizes">
    <Button size="sm">Small</Button>
    <Button size="md">Medium</Button>
    <Button size="lg">Large</Button>
    <Button ariaLabel="Save setup">
      {#snippet left()}<Icon name="save" />{/snippet}
    </Button>
  </Row>
</Section>

<Section component="RunButton" title="RunButton and run control">
  <Row label="Run button">
    <RunButton running={true} onclick={noop} />
    <RunButton running={false} onclick={noop} />
    <RunButton running={false} disabled disabledReason="No device" onclick={noop} />
    <RunButton {running} onclick={() => (running = !running)} />
  </Row>
  <Row label="Run control">
    <Button size="lg">
      {#snippet left()}<Icon name="single" />{/snippet}
      Single
    </Button>
    <Button size="lg" pressed>
      {#snippet left()}<Icon name="single" />{/snippet}
      Single (armed)
    </Button>
    <Button size="lg" disabled disabledReason="Force needs a running or armed scope">
      {#snippet left()}<Icon name="force" />{/snippet}
      Force
    </Button>
  </Row>
</Section>

<Section component="Segmented">
  <Row label="Segmented">
    <Segmented
      ariaLabel="Sweep"
      value={slope}
      onchange={(id) => (slope = id)}
      options={[
        { id: 'auto', label: 'Auto' },
        { id: 'normal', label: 'Normal' },
      ]}
    />
    <Segmented
      ariaLabel="Trigger type"
      value={triggerType}
      onchange={(id) => (triggerType = id)}
      options={[
        { id: 'edge', label: 'Edge' },
        { id: 'pulse', label: 'Pulse' },
        { id: 'video', label: 'Video', disabled: true, disabledReason: 'Video trigger is experimental (illustrative)' },
      ]}
    />
    <Badge kind="experimental" tooltip="Experimental feature">Exp</Badge>
  </Row>
</Section>

<Section component="Stepper">
  <Row label="Stepper">
    <Stepper
      value="500"
      unit="mV/div"
      valueText="500 millivolts per division"
      ariaLabel="Volts per division"
      onstep={noop}
    />
    <Stepper
      value="20"
      unit="mV/div"
      valueText="20 millivolts per division"
      ariaLabel="Volts per division"
      onstep={noop}
    >
      {#snippet badge()}<Badge kind="warning" tooltip="Range not verified (illustrative)">Unverified</Badge>{/snippet}
    </Stepper>
    <Stepper value="1" unit="V/div" off valueText="1 volt per division" ariaLabel="Volts per division" onstep={noop} />
    <Stepper
      value="2"
      unit="ns/div"
      atMax
      valueText="2 nanoseconds per division"
      ariaLabel="Time per division"
      onstep={noop}
    />
  </Row>
</Section>

<Section component="NumberField">
  <Row label="Number field">
    <NumberField ariaLabel="Position" value="-2.00" commit={noop}>
      {#snippet left()}POS{/snippet}
      {#snippet unit()}div{/snippet}
    </NumberField>
    <NumberField ariaLabel="Position, invalid" value="12.0" error="Range -4 to +4 div" commit={noop}>
      {#snippet unit()}div{/snippet}
    </NumberField>
    <NumberField ariaLabel="Position, disabled" value="0.00" disabled disabledReason="Channel off" commit={noop}>
      {#snippet unit()}div{/snippet}
    </NumberField>
  </Row>
</Section>

<style>
  .icon-item {
    display: inline-flex;
    color: var(--ss-color-text-primary);
  }
</style>
