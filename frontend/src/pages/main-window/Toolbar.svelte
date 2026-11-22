<script lang="ts">
  import type { TextScalePercent } from '../../app/text-scale';
  import { textScaleOptions } from '../../app/text-scale';
  import type { ToolbarModel } from '../../app/main-window';
  import Button from '../../ui/button/Button.svelte';
  import Icon from '../../ui/icon/Icon.svelte';
  import Menu from '../../ui/menu/Menu.svelte';
  import RunButton from '../../ui/run-button/RunButton.svelte';
  import Segmented from '../../ui/segmented/Segmented.svelte';
  import StatusChip from '../../ui/status-chip/StatusChip.svelte';
  import Tooltip from '../../ui/tooltip/Tooltip.svelte';
  import AnchoredPopover from './AnchoredPopover.svelte';

  interface Props {
    toolbar: ToolbarModel;
    toolLabels: boolean;
    /** W3 and below: Single, Force and Autoset keep their icons and tooltips but lose their text. */
    runLabels: boolean;
    drawerButton: boolean;
    drawerOpen: boolean;
    textScale: TextScalePercent;
    ontogglerun: () => void;
    ontoggledrawer: () => void;
    ontextscale: (percent: TextScalePercent) => void;
  }

  const {
    toolbar,
    toolLabels,
    runLabels,
    drawerButton,
    drawerOpen,
    textScale,
    ontogglerun,
    ontoggledrawer,
    ontextscale,
  }: Props = $props();

  const noop = () => {};
  const persistenceItems = ['Off', '0.5 s', '1 s', '2 s', 'Infinite'].map((label) => ({ id: label, label }));
  const viewOptions = [
    { id: 'yt', label: 'Yt' },
    { id: 'xy', label: 'XY' },
    { id: 'fft', label: 'FFT', badge: 'SW' },
  ];
  const scaleOptions = textScaleOptions.map((percent) => ({ id: String(percent), label: `${percent} %` }));

  let persistenceAnchor = $state<{ x: number; y: number } | null>(null);
  let settingsOpen = $state(false);

  let persistenceHost: HTMLElement;

  function openPersistence() {
    const box = persistenceHost.getBoundingClientRect();
    persistenceAnchor = { x: box.left, y: box.bottom };
  }

  const selectScale = (id: string) => ontextscale(Number(id) as TextScalePercent);
</script>

{#snippet toolButton(
  label: string,
  icon: 'cursors' | 'math' | 'reference',
  pressed: boolean | undefined,
  title: string,
)}
  <Button {pressed} {title} ariaLabel={toolLabels ? undefined : label}>
    {#snippet left()}<Icon name={icon} />{/snippet}
    {#if toolLabels}{label}{/if}
  </Button>
{/snippet}

<header class="toolbar" role="toolbar" aria-label="Run control and tools" data-region="toolbar">
  {#if runLabels}<div class="brand">SweepScope</div>{/if}

  <div class="group" role="group" aria-label="Run control">
    {#if toolLabels}<span class="ss-caps">Run</span>{/if}
    <RunButton running={toolbar.running} onclick={ontogglerun} />
    <Button
      size="lg"
      title="Single shot: arm once, stop after one trigger (S)"
      ariaLabel={runLabels ? undefined : 'Single'}
    >
      {#snippet left()}<Icon name="single" />{/snippet}
      {#if runLabels}Single{/if}
    </Button>
    <Button size="lg" title="Force one capture now (F)" ariaLabel={runLabels ? undefined : 'Force'}>
      {#snippet left()}<Icon name="force" />{/snippet}
      {#if runLabels}Force{/if}
    </Button>
    <Button
      size="lg"
      title="Pick volts/div and time/div from a short capture (A)"
      ariaLabel={runLabels ? undefined : 'Autoset'}
    >
      {#snippet left()}<Icon name="autoset" />{/snippet}
      {#if runLabels}Autoset{/if}
    </Button>
  </div>

  <div class="group" role="status" aria-live="polite" aria-label="Acquisition state">
    {#each toolbar.chips as chip (chip.id)}
      {#if chip.tooltip}
        <Tooltip text={chip.tooltip}>
          <StatusChip kind={chip.kind} filled={chip.filled} dim={chip.dim}>{chip.text}</StatusChip>
        </Tooltip>
      {:else}
        <StatusChip kind={chip.kind} filled={chip.filled} dim={chip.dim}>{chip.text}</StatusChip>
      {/if}
    {/each}
  </div>

  <div class="group" role="group" aria-label="View">
    {#if toolLabels}<span class="ss-caps">View</span>{/if}
    <Segmented value={toolbar.view} options={viewOptions} onchange={noop} ariaLabel="View" />
    <span class="anchor" bind:this={persistenceHost}>
      <Button title="Persistence decay: Off, 0.5 s, 1 s, 2 s, infinite" onclick={openPersistence}>
        {toolbar.persistence}
        {#snippet right()}<Icon name="chevron" />{/snippet}
      </Button>
    </span>
  </div>

  <div class="grow"></div>

  <div class="group" role="group" aria-label="Tools">
    {@render toolButton('Cursors', 'cursors', toolbar.tools.cursors, 'Cursors (C)')}
    {@render toolButton('Math', 'math', toolbar.tools.math, 'Math channel (A+B, A-B, AxB)')}
    {@render toolButton('Ref', 'reference', toolbar.tools.reference, 'Store or show a reference waveform')}
  </div>

  <div class="group" role="group" aria-label="Data">
    <Button title="Save setup or capture" ariaLabel={toolLabels ? undefined : 'Save'}>
      {#snippet left()}<Icon name="save" />{/snippet}
      {#if toolLabels}Save{/if}
      {#snippet right()}<Icon name="chevron" />{/snippet}
    </Button>
    <Button title="Screenshot to PNG (Ctrl+P)" ariaLabel="Screenshot">
      {#snippet left()}<Icon name="screenshot" />{/snippet}
    </Button>
    {#if drawerButton}
      <Button
        title="Cursors and measurements drawer"
        ariaLabel="Measurements drawer"
        pressed={drawerOpen}
        onclick={ontoggledrawer}
      >
        {#snippet left()}<Icon name="cursors" />{/snippet}
      </Button>
    {/if}
    <span class="anchor">
      <Button
        title="Settings"
        ariaLabel="Settings"
        pressed={settingsOpen}
        onclick={() => (settingsOpen = !settingsOpen)}
      >
        {#snippet left()}<Icon name="settings" />{/snippet}
      </Button>
      {#if settingsOpen}
        <AnchoredPopover label="Settings" placement="below-end" onclose={() => (settingsOpen = false)}>
          {#snippet title()}Settings{/snippet}
          <span class="setting">
            <span class="ss-caps">Text size</span>
            <Segmented value={String(textScale)} options={scaleOptions} onchange={selectScale} ariaLabel="Text size" />
          </span>
        </AnchoredPopover>
      {/if}
    </span>
  </div>
</header>

{#if persistenceAnchor}
  <Menu
    items={persistenceItems}
    open={true}
    anchor={persistenceAnchor}
    onselect={() => (persistenceAnchor = null)}
    onclose={() => (persistenceAnchor = null)}
  />
{/if}

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--ss-space-4);
    padding: 0 var(--ss-space-4);
    min-block-size: var(--ss-size-toolbar);
    background: var(--ss-color-bg-panel);
    border-bottom: var(--ss-border-width) solid var(--ss-color-border);
  }
  .brand {
    font-weight: var(--ss-font-weight-bold);
    letter-spacing: var(--ss-letter-spacing-caps);
    font-size: var(--ss-font-size-sm);
    color: var(--ss-color-text-secondary);
    text-transform: uppercase;
    padding-inline-end: var(--ss-space-4);
    border-inline-end: var(--ss-border-width) solid var(--ss-color-border);
  }
  .group {
    display: flex;
    align-items: center;
    gap: var(--ss-space-2);
  }
  .group + .group {
    padding-inline-start: var(--ss-space-4);
    border-inline-start: var(--ss-border-width) solid var(--ss-color-border-subtle);
  }
  .grow {
    flex: 1;
  }
  .anchor {
    position: relative;
    display: inline-flex;
  }
  .setting {
    display: flex;
    align-items: center;
    gap: var(--ss-space-4);
  }
</style>
