<script module lang="ts">
  import type { CalibrationDialogModel } from '../../app/calibration';

  export interface CalibrationDialogProps {
    dialog: CalibrationDialogModel;
    onstart: () => void;
    oncalibrate: () => void;
    /** Accepts the rejected reading; offered only while the dialog says it may. */
    oncontinue: () => void;
    /** Cancels before Done, closes at Done and after a failure. */
    onclose: () => void;
  }
</script>

<script lang="ts">
  import { CALIBRATION_STEPS } from '../../app/calibration';
  import Button from '../../ui/button/Button.svelte';
  import { Dialog } from '../../ui/dialog';
  import { StepChips } from '../../ui/step-chips';
  import type { StepChipProps } from '../../ui/step-chips';
  import ChannelStep from './calibration-steps/ChannelStep.svelte';
  import DoneStep from './calibration-steps/DoneStep.svelte';
  import FailedStep from './calibration-steps/FailedStep.svelte';
  import IntroStep from './calibration-steps/IntroStep.svelte';
  import MeasuringStep from './calibration-steps/MeasuringStep.svelte';
  import NotShortedStep from './calibration-steps/NotShortedStep.svelte';
  import { channelNumbers } from './calibration-steps/channels';

  const { dialog, onstart, oncalibrate, oncontinue, onclose }: CalibrationDialogProps = $props();

  const bodyId = $props.id();
  // Read before the dialog takes the focus, so it can go back to whatever opened it (the status tag).
  const opener = document.activeElement;
  let stepRoot: HTMLElement;

  const channel = $derived(dialog.channel ?? 1);
  const showChips = $derived(dialog.step !== 'intro');
  const stepKey = $derived(`${dialog.step}:${dialog.channel}`);

  function chipState(number: number): StepChipProps['state'] {
    if (dialog.step === 'done' || dialog.doneChannels[number - 1]) return 'done';
    return dialog.channel === number ? 'current' : 'todo';
  }

  const chipName = (number: number, state: StepChipProps['state']) =>
    `Channel ${number}, ${state === 'todo' ? 'to do' : state}`;

  const chips = $derived(
    channelNumbers.map((number): StepChipProps => {
      const state = chipState(number);
      return { label: String(number), state, color: `var(--ss-color-ch${number})`, name: chipName(number, state) };
    }),
  );

  // On open and on every step the focus goes to the step heading, never to a primary action (Enter must not start a measurement).
  $effect(() => {
    void stepKey;
    stepRoot.querySelector<HTMLElement>('[data-step-focus]')?.focus();
  });

  $effect(() => () => {
    if (opener instanceof HTMLElement && opener !== document.body) {
      queueMicrotask(() => {
        if (opener.isConnected) opener.focus();
      });
    }
  });
</script>

{#snippet primary(label: string, onclick: (() => void) | undefined)}
  <span class="primary"><Button {onclick} disabled={onclick === undefined}>{label}</Button></span>
{/snippet}

<Dialog open modal wide {onclose} describedby={bodyId}>
  {#snippet title()}Zero calibration{/snippet}
  {#snippet meta()}Step {dialog.stepNumber} of {CALIBRATION_STEPS}{/snippet}
  {#snippet strip()}
    {#if showChips}<StepChips caption="Channels" items={chips} />{/if}
  {/snippet}
  {#snippet body()}
    <div class="step" id={bodyId} bind:this={stepRoot}>
      {#key stepKey}
        {#if dialog.step === 'intro'}
          <IntroStep calibrated={dialog.calibrated} storedAt={dialog.storedAt} />
        {:else if dialog.step === 'channel'}
          <ChannelStep {channel} />
        {:else if dialog.step === 'measuring'}
          <MeasuringStep {channel} progress={dialog.progress} />
        {:else if dialog.step === 'notShorted'}
          <NotShortedStep {channel} measured={dialog.notShorted} canContinue={dialog.canContinue} />
        {:else if dialog.step === 'done'}
          <DoneStep table={dialog.table} overridden={dialog.overridden} />
        {:else}
          <FailedStep failure={dialog.failure ?? ''} />
        {/if}
      {/key}
    </div>
  {/snippet}
  {#snippet actions()}
    {#if dialog.step === 'done'}
      {@render primary('Done', onclose)}
    {:else if dialog.step === 'failed'}
      {@render primary('Close', onclose)}
    {:else}
      {#if dialog.step === 'measuring'}<span class="hint">Cancel stores nothing.</span>{/if}
      <Button onclick={onclose}>Cancel</Button>
      {#if dialog.step === 'intro'}
        {@render primary('Start', onstart)}
      {:else if dialog.step === 'channel'}
        {@render primary('Calibrate', oncalibrate)}
      {:else if dialog.step === 'notShorted'}
        {#if dialog.canContinue}<Button onclick={oncontinue}>Continue</Button>{/if}
        {@render primary('Try again', oncalibrate)}
      {:else}
        {@render primary('Measuring...', undefined)}
      {/if}
    {/if}
  {/snippet}
</Dialog>

<style>
  .step {
    display: grid;
    gap: var(--ss-space-4);
    align-content: start;
    min-block-size: 15rem;
  }
  .primary :global(.btn:not(.disabled)) {
    background: var(--ss-color-bg-selected);
    border-color: var(--ss-color-accent);
    font-weight: var(--ss-font-weight-bold);
  }
  .hint {
    margin-inline-end: auto;
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-muted);
  }
</style>
