<script lang="ts">
  import Note from './Note.svelte';
  import StepHeading from './StepHeading.svelte';

  const { channel, progress }: { channel: number; progress: number | null } = $props();
  const percent = $derived(progress === null ? null : Math.round(Math.min(1, Math.max(0, progress)) * 100));
</script>

<StepHeading>Channel {channel}: measuring</StepHeading>
<Note kind="info" role="status">Measuring CH{channel}, keep the input shorted and still</Note>
<div
  class="bar"
  class:indeterminate={percent === null}
  role="progressbar"
  aria-valuemin="0"
  aria-valuemax="100"
  aria-valuenow={percent ?? undefined}
  aria-label="Measuring channel {channel}"
>
  <i style:inline-size={percent === null ? undefined : `${percent}%`}></i>
</div>
{#if percent !== null}<div class="readline">{percent} %</div>{/if}

<style>
  .bar {
    block-size: 0.5rem;
    background: var(--ss-color-bg-inset);
    border: var(--ss-border-width) solid var(--ss-color-border);
    overflow: hidden;
  }
  i {
    display: block;
    block-size: 100%;
    background: var(--ss-color-accent);
  }
  .indeterminate i {
    inline-size: 30%;
  }
  @media (prefers-reduced-motion: no-preference) {
    .indeterminate i {
      animation: sweep var(--ss-motion-normal) var(--ss-motion-easing) infinite alternate;
    }
  }
  @keyframes sweep {
    from {
      margin-inline-start: 0;
    }
    to {
      margin-inline-start: 70%;
    }
  }
  .readline {
    font-family: var(--ss-font-mono);
    font-size: var(--ss-font-size-xs);
  }
</style>
