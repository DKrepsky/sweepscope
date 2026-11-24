<script lang="ts">
  import Note from './Note.svelte';
  import StepHeading from './StepHeading.svelte';

  const { channel, measured, canContinue }: { channel: number; measured: string[]; canContinue: boolean } = $props();
</script>

<StepHeading>Channel {channel}: short the input</StepHeading>
<Note kind="error" role="alert">
  <b>The CH{channel} input does not look shorted. Nothing was stored for this channel.</b>
  <br />
  {#each measured as line (line)}
    {line}
  {/each}
  Check the connection between probe tip and ground clip, then try again.
</Note>
<ol>
  <li>Join the probe tip to its ground clip on <b>CH{channel}</b>.</li>
  <li>Click <b>Try again</b>.</li>
</ol>
{#if canContinue}
  <Note kind="warning">Continue keeps the measured offsets for CH{channel}, use it only if the input is shorted</Note>
{/if}

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
