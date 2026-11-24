<script lang="ts">
  import type { GapModel } from '../../app/main-window';

  interface Props {
    gaps: GapModel[];
    /** Said politely when it changes, for example `Data gap of 1.2 s`. */
    announcement: string;
  }

  const { gaps, announcement }: Props = $props();

  const percent = (fraction: number) => `${fraction * 100}%`;
</script>

<div class="gaps" data-testid="roll-gaps">
  {#each gaps as gap, index (index)}
    <div class="band" style:left={percent(gap.from)} style:width={percent(gap.to - gap.from)}>
      <span class="label ss-num">{gap.label}</span>
    </div>
  {/each}
</div>
<p class="live" aria-live="polite" role="status">{announcement}</p>

<style>
  .gaps {
    position: absolute;
    inset: 0;
    overflow: hidden;
    pointer-events: none;
  }
  .band {
    position: absolute;
    inset-block: 0;
    background: var(--ss-color-gap-fill);
    border-inline: var(--ss-border-width) dashed var(--ss-color-status-warning);
  }
  .label {
    position: absolute;
    inset-block-start: var(--ss-space-2);
    inset-inline-start: var(--ss-space-2);
    font-size: var(--ss-font-size-2xs);
    color: var(--ss-color-status-warning);
    white-space: nowrap;
  }
  .live {
    position: absolute;
    inline-size: var(--ss-border-width);
    block-size: var(--ss-border-width);
    margin: 0;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }
</style>
