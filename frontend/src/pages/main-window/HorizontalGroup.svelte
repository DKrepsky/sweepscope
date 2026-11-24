<script lang="ts">
  import type {
    AxisOption,
    ChannelNumber,
    HorizontalModel,
    MainWindowActions,
    SpectrumScaleName,
    SpectrumWindowName,
    ViewsModel,
  } from '../../app/main-window';
  import Badge from '../../ui/badge/Badge.svelte';
  import Button from '../../ui/button/Button.svelte';
  import Menu from '../../ui/menu/Menu.svelte';
  import Segmented from '../../ui/segmented/Segmented.svelte';
  import { channelStyle } from './channel-style';
  import DockGroup from './DockGroup.svelte';
  import ReasonStepper from './ReasonStepper.svelte';

  interface Props {
    horizontal: HorizontalModel;
    views: ViewsModel;
    actions: MainWindowActions;
  }

  const { horizontal, views, actions }: Props = $props();

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

  const AXIS_LABELS = { x: 'X axis', y: 'Y axis' } as const;
  const axisOptions = (options: AxisOption[]) =>
    options.map(({ channel, availability }) => ({
      id: String(channel),
      label: String(channel),
      disabled: !availability.enabled,
      disabledReason: availability.reason,
    }));
  const chooseAxis = (axis: 'x' | 'y', id: string) => actions.setXyAxis(axis, Number(id) as ChannelNumber);

  const SCALES = [
    { id: 'dbv', label: 'dBV' },
    { id: 'linear', label: 'Linear' },
  ];
  const scaleOptions = $derived(
    SCALES.map((scale) => ({
      ...scale,
      disabled: !views.fft.available.enabled,
      disabledReason: views.fft.available.reason,
    })),
  );
  const chooseScale = (id: string) => actions.setSpectrumScale(id as SpectrumScaleName);

  let windowHost = $state<HTMLElement>();
  let windowAnchor = $state<{ x: number; y: number } | null>(null);
  const windowItems = $derived(
    views.fft.windows.map(({ value, label }) => ({ id: value, label, checked: value === views.fft.window })),
  );
  function openWindowMenu() {
    const box = windowHost?.getBoundingClientRect();
    if (!box) return;
    windowAnchor = { x: box.left, y: box.bottom };
  }
  function chooseWindow(id: string) {
    windowAnchor = null;
    actions.setSpectrumWindow(id as SpectrumWindowName);
  }

  const timeTip = $derived(
    views.tab === 'yt' ? undefined : 'Sets the sample rate, and so the span, the bin spacing and the points per cycle.',
  );
  const note = $derived(
    views.tab === 'xy' ? views.xy.readline : views.tab === 'fft' ? views.fft.readline : horizontal.rateNote,
  );
</script>

<DockGroup title="Horizontal" region="horizontal">
  <div class="line">
    <span class="ss-caps label" title={timeTip}>Time/div</span>
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
  {#if views.tab === 'yt'}
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
  {:else if views.tab === 'xy'}
    {#each ['x', 'y'] as const as axis (axis)}
      <div class="line">
        <span class="ss-caps label">{AXIS_LABELS[axis]}</span>
        <Segmented
          value={String(views.xy[axis])}
          options={axisOptions(axis === 'x' ? views.xy.xOptions : views.xy.yOptions)}
          onchange={(id) => chooseAxis(axis, id)}
          ariaLabel="{AXIS_LABELS[axis]} channel"
        >
          {#snippet option({ id })}
            <span class="axis" style={channelStyle(Number(id) as ChannelNumber)}>{id}</span>
          {/snippet}
        </Segmented>
      </div>
    {/each}
  {:else}
    <div class="line">
      <span class="ss-caps label">Scale</span>
      <Segmented value={views.fft.scale} options={scaleOptions} onchange={chooseScale} ariaLabel="Spectrum scale" />
    </div>
    <div class="line">
      <span class="ss-caps label">Window</span>
      <span bind:this={windowHost}>
        <Button
          size="sm"
          ariaLabel="Window: {views.fft.windowLabel}"
          aria-haspopup="menu"
          disabled={!views.fft.available.enabled}
          disabledReason={views.fft.available.reason}
          onclick={openWindowMenu}
        >
          {views.fft.windowShort} ▾
        </Button>
      </span>
      <Badge kind="software" tooltip="The window is applied in software">SW</Badge>
    </div>
  {/if}
  <div class="note ss-num" title="Slot: effective sample rate. Value is illustrative.">{note}</div>
</DockGroup>

{#if windowAnchor}
  <Menu
    items={windowItems}
    open={true}
    anchor={windowAnchor}
    onselect={chooseWindow}
    onclose={() => (windowAnchor = null)}
  />
{/if}

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
  .axis {
    color: var(--chc);
    font-weight: var(--ss-font-weight-bold);
  }
  .note {
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-secondary);
  }
</style>
