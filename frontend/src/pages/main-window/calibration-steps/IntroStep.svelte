<script lang="ts">
  import Note from './Note.svelte';
  import StepHeading from './StepHeading.svelte';

  const { calibrated, storedAt }: { calibrated: boolean; storedAt: string | undefined } = $props();
</script>

<StepHeading>Why calibrate</StepHeading>
{#if calibrated}
  <Note kind="good">
    <b>This device is calibrated</b>
    {#if storedAt}<br />Last calibrated at {storedAt}{/if}
  </Note>
{/if}
<p>
  Calibration measures the zero error of each input while it is shorted and subtracts it, so small voltages read
  correctly.
</p>
<ul>
  <li>You short each of the 4 inputs in turn</li>
  <li>Acquisition is paused while this dialog is open</li>
  <li>Nothing is stored until you finish, and the factory calibration is never changed</li>
</ul>

<style>
  p {
    margin: 0;
  }
  ul {
    margin: 0;
    padding-inline-start: 1.2rem;
    display: grid;
    gap: var(--ss-space-3);
  }
</style>
