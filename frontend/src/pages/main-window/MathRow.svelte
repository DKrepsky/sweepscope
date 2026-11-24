<script lang="ts">
  import type {
    ChannelModel,
    ChannelNumber,
    MainWindowActions,
    MathModel,
    MathOperationName,
  } from '../../app/main-window';
  import Badge from '../../ui/badge/Badge.svelte';
  import Button from '../../ui/button/Button.svelte';
  import Icon from '../../ui/icon/Icon.svelte';
  import Menu from '../../ui/menu/Menu.svelte';
  import { menuTriggerAttributes } from '../../ui/menu';
  import NumberField from '../../ui/number-field/NumberField.svelte';
  import ReasonStepper from './ReasonStepper.svelte';

  interface Props {
    math: MathModel;
    channels: ChannelModel[];
    actions: MainWindowActions;
  }

  const { math, channels, actions }: Props = $props();

  type Picker = 'operation' | 'a' | 'b';

  const OPERATIONS: { id: MathOperationName; label: string }[] = [
    { id: 'add', label: 'A+B' },
    { id: 'subtract', label: 'A−B' },
    { id: 'multiply', label: 'A×B' },
    { id: 'invertA', label: 'Invert A' },
  ];

  const row = $derived(math.row);
  const blocked = $derived(row.availability.enabled === false);
  const dim = $derived(!row.enabled || blocked || row.status === 'unavailable');
  const reason = $derived(blocked ? row.availability.reason : undefined);
  const needsB = $derived(row.operation !== 'invertA');
  const operationLabel = $derived(OPERATIONS.find((o) => o.id === row.operation)?.label ?? row.label);
  const sourceBText = $derived(needsB ? `B ${row.sourceB ?? '-'}` : 'B -');
  const statusText = $derived(row.status === 'mixed' ? 'Mixed' : 'Unavailable');

  let open = $state<{ picker: Picker; anchor: { x: number; y: number } } | null>(null);
  const hosts: Partial<Record<Picker, HTMLElement>> = {};

  function show(picker: Picker) {
    const box = hosts[picker]?.getBoundingClientRect();
    if (box) open = { picker, anchor: { x: box.left, y: box.bottom } };
  }

  const sourceItems = $derived(
    channels.map((channel) => ({
      id: String(channel.number),
      label: `CH${channel.number}`,
      disabled: !channel.enabled,
      disabledReason: `Channel ${channel.number} is off`,
    })),
  );
  const items = $derived(open?.picker === 'operation' ? OPERATIONS : sourceItems);

  function choose(id: string) {
    const picker = open?.picker;
    open = null;
    if (picker === 'operation') actions.setMathOperation(id as MathOperationName);
    else if (picker) actions.setMathSource(picker, Number(id) as ChannelNumber);
  }

  const toggleReason = $derived(reason);
  function toggle() {
    if (!toggleReason) actions.setMathEnabled(!row.enabled);
  }
</script>

{#snippet picker(kind: Picker, text: string, ariaLabel: string, disabled: boolean, colour?: ChannelNumber)}
  <span class="host" bind:this={hosts[kind]} style:color={colour ? `var(--ss-color-ch${colour})` : undefined}>
    <Button
      {disabled}
      disabledReason={reason}
      {ariaLabel}
      title={ariaLabel}
      onclick={() => show(kind)}
      {...menuTriggerAttributes(open?.picker === kind)}
    >
      {text}
      {#snippet right()}<Icon name="chevron" />{/snippet}
    </Button>
  </span>
{/snippet}

<div class="row" data-math data-enabled={row.enabled} data-status={row.status}>
  <button
    type="button"
    class="toggle focus-ring"
    class:outline={!row.enabled}
    aria-pressed={row.enabled}
    aria-disabled={toggleReason ? 'true' : undefined}
    title={toggleReason ?? 'Math channel on'}
    aria-label="M, math channel enabled"
    onclick={toggle}
  >
    M
  </button>
  <ReasonStepper
    value={row.scale.value}
    unit={row.scale.unit}
    smaller={row.scale.step.faster}
    larger={row.scale.step.slower}
    onstep={actions.stepMathScale}
    onreset={actions.resetMathScale}
    off={dim}
    {reason}
    valueText="{row.scale.value} {row.scale.unit}"
    ariaLabel="Math scale"
  />
  <NumberField
    value={row.position.value}
    commit={(text) => actions.setMathPosition(Number(text.replace('−', '-').replace(',', '.')))}
    disabled={dim}
    disabledReason={reason}
    ariaLabel="Math vertical position in divisions"
  >
    {#snippet left()}<small>POS</small>{/snippet}
    {#snippet unit()}<small>{row.position.unit}</small>{/snippet}
  </NumberField>
  {@render picker('operation', operationLabel, `${operationLabel}, math operation`, blocked)}
  {@render picker('a', `A ${row.sourceA}`, `A ${row.sourceA}, math source A`, blocked, row.sourceA)}
  {@render picker('b', sourceBText, `${sourceBText}, math source B`, blocked || !needsB, row.sourceB)}
  <Badge kind="software" tooltip="Computed in software">SW</Badge>
  <span class="status">
    {#if row.status !== 'ok'}
      <Badge kind="warning" tooltip={row.statusDetail ?? statusText}>{statusText}</Badge>
    {/if}
  </span>
</div>

{#if open}
  <Menu {items} open={true} anchor={open.anchor} onselect={choose} onclose={() => (open = null)} />
{/if}

<style>
  .row {
    display: grid;
    grid-template-columns: subgrid;
    grid-column: 1 / -1;
    align-items: center;
    padding: var(--ss-space-1) var(--ss-space-2) var(--ss-space-1) 0;
    border-inline-start: calc(var(--ss-border-width) * 3) solid var(--ss-color-math);
    white-space: nowrap;
  }
  .toggle {
    display: grid;
    place-items: center;
    inline-size: var(--ss-size-control-md);
    block-size: var(--ss-size-control-md);
    margin-inline-start: var(--ss-space-2);
    font-family: var(--ss-font-mono);
    font-weight: var(--ss-font-weight-bold);
    color: var(--ss-color-text-on-accent);
    background: var(--ss-color-math);
    border: var(--ss-border-width) solid var(--ss-color-math);
  }
  .toggle.outline {
    color: var(--ss-color-math);
    background: var(--ss-color-bg-inset);
  }
  .host {
    display: inline-flex;
  }
  .status {
    justify-self: end;
  }
</style>
