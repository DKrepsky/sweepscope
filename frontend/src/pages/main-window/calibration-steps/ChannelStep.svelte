<script lang="ts">
  import Note from './Note.svelte';
  import ProbePicture from './ProbePicture.svelte';
  import StepHeading from './StepHeading.svelte';

  const { channel }: { channel: number } = $props();
</script>

<StepHeading>Channel {channel}: short the input</StepHeading>
<ol>
  <li><b>Disconnect</b> the probe from the circuit. Never short a live circuit.</li>
  <li>
    <b>Short the CH{channel} input:</b> touch the probe tip to its ground clip, or join the BNC centre pin to its shell.
  </li>
  <li>Click <b>Calibrate</b> when ready, and keep the input still.</li>
</ol>
<ProbePicture />
<Note kind="info">An open input is not zero: it picks up the neighbouring channels.</Note>

<style>
  ol {
    margin: 0;
    padding-inline-start: 1.2rem;
    display: grid;
    gap: var(--ss-space-3);
  }
  b {
    color: var(--ss-color-text-primary);
    font-weight: var(--ss-font-weight-medium);
  }
</style>
