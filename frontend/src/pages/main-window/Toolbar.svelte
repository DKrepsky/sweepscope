<script lang="ts">
  import type { TextScalePercent } from '../../app/text-scale';
  import { textScaleOptions } from '../../app/text-scale';
  import type { FileActions } from '../../app/files';
  import type {
    Availability,
    AutosetModel,
    ChipKind,
    PersistenceModel,
    PersistenceValue,
    RecordingModel,
    ToolbarModel,
    ViewTab,
    ViewsModel,
  } from '../../app/main-window';
  import Button from '../../ui/button/Button.svelte';
  import Icon from '../../ui/icon/Icon.svelte';
  import Menu from '../../ui/menu/Menu.svelte';
  import { menuTriggerAttributes } from '../../ui/menu';
  import RunButton from '../../ui/run-button/RunButton.svelte';
  import Segmented from '../../ui/segmented/Segmented.svelte';
  import StatusChip from '../../ui/status-chip/StatusChip.svelte';
  import Tooltip from '../../ui/tooltip/Tooltip.svelte';
  import AnchoredPopover from './AnchoredPopover.svelte';

  interface Props {
    toolbar: ToolbarModel;
    views: ViewsModel;
    persistence: PersistenceModel;
    autoset: AutosetModel;
    recording: RecordingModel;
    toolLabels: boolean;
    /** W3 and below: Single, Force and Autoset keep their icons and tooltips but lose their text. */
    runLabels: boolean;
    drawerButton: boolean;
    drawerOpen: boolean;
    textScale: TextScalePercent;
    ontogglerun: () => void;
    onforce: () => void;
    onarmsingle: () => void;
    onautoset: () => void;
    onsetview: (view: ViewTab) => void;
    onsetpersistence: (value: PersistenceValue) => void;
    onclearpersistence: () => void;
    ontogglemath: () => void;
    onstartrecording: () => void;
    onstoprecording: () => void;
    ontogglecursors: () => void;
    ontoggledrawer: () => void;
    ontextscale: (percent: TextScalePercent) => void;
    onfile: (action: keyof FileActions) => void;
  }

  const {
    toolbar,
    views,
    persistence,
    autoset,
    recording,
    toolLabels,
    runLabels,
    drawerButton,
    drawerOpen,
    textScale,
    ontogglerun,
    onforce,
    onarmsingle,
    onautoset,
    onsetview,
    onsetpersistence,
    onclearpersistence,
    ontogglemath,
    onstartrecording,
    onstoprecording,
    ontogglecursors,
    ontoggledrawer,
    ontextscale,
    onfile,
  }: Props = $props();

  const AUTOSET_RUNNING = 'Auto-setup is running';
  const CLEAR_PERSISTENCE = 'clear';
  const PERSISTENCE_TIP =
    'Persistence: overlays successive frames. Its update rate is the capture rate shown in the status bar.';
  const runOff = $derived(toolbar.availability?.enabled === false);
  const forceOff = $derived(toolbar.force?.enabled === false);
  const single = $derived(toolbar.single);
  const singleOff = $derived(single?.availability.enabled === false);
  const autosetOff = $derived(autoset.busy || !autoset.available.enabled);
  const persistenceLabel = $derived(persistence.options.find((o) => o.value === persistence.value)?.label);
  const persistenceItems = $derived([
    ...persistence.options.map((option) => ({
      id: option.value,
      label: option.label,
      checked: option.value === persistence.value,
      radio: true,
    })),
    { id: CLEAR_PERSISTENCE, label: 'Clear persistence', separatorBefore: true },
  ]);
  const viewOptions = $derived(
    (
      [
        ['yt', 'Yt', undefined],
        ['xy', 'XY', undefined],
        ['fft', 'FFT', 'SW'],
      ] as const
    ).map(([id, label, badge]) => ({
      id,
      label,
      badge,
      disabled: !views.tabAvailability[id].enabled,
      disabledReason: views.tabAvailability[id].reason,
    })),
  );
  const chipKind = (kind: ChipKind) => (kind === 'roll' ? 'info' : kind);
  const scaleOptions = textScaleOptions.map((percent) => ({ id: String(percent), label: `${percent} %` }));

  const fileItems = $derived(
    (
      [
        ['saveSetup', 'Save setup...', toolbar.files.saveSetup],
        ['loadSetup', 'Load setup...', toolbar.files.loadSetup],
        ['exportCsv', 'Export CSV...', toolbar.files.exportCsv],
        ['exportRaw', 'Export raw capture...', toolbar.files.exportRaw],
        ['saveScreenshot', 'Save screenshot...', toolbar.files.screenshot],
      ] as const
    ).map(([id, label, availability]) => ({
      id,
      label,
      disabled: !availability.enabled,
      disabledReason: availability.reason,
    })),
  );

  let persistenceAnchor = $state<{ x: number; y: number } | null>(null);
  let fileAnchor = $state<{ x: number; y: number } | null>(null);
  let fileHost: HTMLElement;
  let settingsOpen = $state(false);

  let persistenceHost: HTMLElement;

  function openPersistence() {
    const box = persistenceHost.getBoundingClientRect();
    persistenceAnchor = { x: box.left, y: box.bottom };
  }

  function openFileMenu() {
    const box = fileHost.getBoundingClientRect();
    fileAnchor = { x: box.left, y: box.bottom };
  }

  function selectPersistence(id: string) {
    persistenceAnchor = null;
    if (id === CLEAR_PERSISTENCE) onclearpersistence();
    else onsetpersistence(id as PersistenceValue);
  }

  function selectFile(id: string) {
    fileAnchor = null;
    onfile(id as keyof FileActions);
  }

  const selectScale = (id: string) => ontextscale(Number(id) as TextScalePercent);
</script>

{#snippet toolButton(
  label: string,
  icon: 'cursors' | 'math' | 'reference',
  pressed: boolean | undefined,
  title: string,
  onclick?: () => void,
  availability?: Availability,
)}
  <Button
    {pressed}
    {title}
    ariaLabel={toolLabels ? undefined : label}
    {onclick}
    disabled={availability?.enabled === false}
    disabledReason={availability?.reason}
  >
    {#snippet left()}<Icon name={icon} />{/snippet}
    {#if toolLabels}{label}{/if}
  </Button>
{/snippet}

<header class="toolbar" role="toolbar" aria-label="Run control and tools" data-region="toolbar">
  {#if runLabels}<div class="brand">SweepScope</div>{/if}

  <div class="group" role="group" aria-label="Run control">
    {#if toolLabels}<span class="ss-caps">Run</span>{/if}
    <RunButton
      running={toolbar.running}
      onclick={ontogglerun}
      disabled={runOff}
      disabledReason={toolbar.availability?.reason}
    />
    <Button
      size="lg"
      title="Single shot: arm once, stop after one trigger. Press again to cancel."
      ariaLabel={runLabels ? undefined : 'Single'}
      pressed={single?.armed ?? false}
      onclick={single?.armed ? ontogglerun : onarmsingle}
      disabled={singleOff}
      disabledReason={single?.availability.reason}
    >
      {#snippet left()}<Icon name="single" />{/snippet}
      {#if runLabels}Single{/if}
    </Button>
    <Button
      size="lg"
      title="Force one capture now (F)"
      ariaLabel={runLabels ? undefined : 'Force'}
      onclick={onforce}
      disabled={forceOff}
      disabledReason={toolbar.force?.reason}
    >
      {#snippet left()}<Icon name="force" />{/snippet}
      {#if runLabels}Force{/if}
    </Button>
    <Button
      size="lg"
      title="Pick volts/div and time/div from a short capture (A)"
      ariaLabel={runLabels ? undefined : 'Autoset'}
      busy={autoset.busy}
      onclick={onautoset}
      disabled={autosetOff}
      disabledReason={autoset.busy ? AUTOSET_RUNNING : autoset.available.reason}
    >
      {#snippet left()}<Icon name="autoset" />{/snippet}
      {#if runLabels}Autoset{/if}
    </Button>
  </div>

  <div class="group" role="status" aria-live="polite" aria-label="Acquisition state">
    {#each toolbar.chips as chip (chip.id)}
      {#if chip.tooltip}
        <Tooltip text={chip.tooltip}>
          <StatusChip kind={chipKind(chip.kind)} filled={chip.filled} dim={chip.dim}>{chip.text}</StatusChip>
        </Tooltip>
      {:else}
        <StatusChip kind={chipKind(chip.kind)} filled={chip.filled} dim={chip.dim}>{chip.text}</StatusChip>
      {/if}
    {/each}
  </div>

  <div class="group" role="group" aria-label="View">
    {#if toolLabels}<span class="ss-caps">View</span>{/if}
    <Segmented value={views.tab} options={viewOptions} onchange={(id) => onsetview(id as ViewTab)} ariaLabel="View" />
    <span class="anchor" bind:this={persistenceHost}>
      <Button
        title={PERSISTENCE_TIP}
        pressed={persistence.value !== 'off'}
        onclick={openPersistence}
        disabled={!persistence.available.enabled}
        disabledReason={persistence.available.reason}
        {...menuTriggerAttributes(persistenceAnchor !== null)}
      >
        Persist: {persistenceLabel}
        {#snippet right()}<Icon name="chevron" />{/snippet}
      </Button>
    </span>
  </div>

  <div class="grow"></div>

  <div class="group" role="group" aria-label="Tools">
    {@render toolButton(
      'Cursors',
      'cursors',
      toolbar.tools.cursors,
      'Cursors (C)',
      ontogglecursors,
      toolbar.cursorsAvailability,
    )}
    {@render toolButton(
      'Math',
      'math',
      toolbar.tools.math,
      'Math channel (A+B, A-B, AxB)',
      ontogglemath,
      toolbar.mathAvailability,
    )}
    {@render toolButton('Ref', 'reference', toolbar.tools.reference, 'Store or show a reference waveform')}
  </div>

  <div class="group" role="group" aria-label="Data">
    <Button
      title="Record roll data to a file"
      ariaLabel={runLabels ? undefined : 'Record'}
      tone={recording.active ? 'record' : undefined}
      pressed={recording.active}
      onclick={recording.active ? onstoprecording : onstartrecording}
      disabled={!recording.available.enabled && !recording.active}
      disabledReason={recording.available.reason}
    >
      {#snippet left()}<Icon name="record" />{/snippet}
      {#if runLabels}{recording.active ? 'Stop rec' : 'Record'}{/if}
    </Button>
    <span class="anchor" bind:this={fileHost}>
      <Button
        title="Save or load a setup, export the capture, save a screenshot"
        ariaLabel={toolLabels ? undefined : 'File'}
        onclick={openFileMenu}
        {...menuTriggerAttributes(fileAnchor !== null)}
      >
        {#snippet left()}<Icon name="save" />{/snippet}
        {#if toolLabels}File{/if}
        {#snippet right()}<Icon name="chevron" />{/snippet}
      </Button>
    </span>
    <Button
      title="Screenshot to PNG (Ctrl+P)"
      ariaLabel="Screenshot"
      onclick={() => onfile('saveScreenshot')}
      disabled={!toolbar.files.screenshot.enabled}
      disabledReason={toolbar.files.screenshot.reason}
    >
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
  </div>

  <div class="pinned">
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

{#if fileAnchor}
  <Menu items={fileItems} open={true} anchor={fileAnchor} onselect={selectFile} onclose={() => (fileAnchor = null)} />
{/if}

{#if persistenceAnchor}
  <Menu
    items={persistenceItems}
    open={true}
    anchor={persistenceAnchor}
    onselect={selectPersistence}
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
  /* Stays at the right edge of the viewport when the window is narrower than the content floor, so the text size is always reachable. */
  .pinned {
    position: sticky;
    inset-inline-end: 0;
    z-index: var(--ss-z-overlay);
    display: flex;
    align-items: center;
    align-self: stretch;
    padding-inline: var(--ss-space-4);
    margin-inline-end: calc(-1 * var(--ss-space-4));
    background: var(--ss-color-bg-panel);
    border-inline-start: var(--ss-border-width) solid var(--ss-color-border-subtle);
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
