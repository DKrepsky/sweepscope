<script lang="ts">
  import type { CalibrationDialogModel } from '../../../app/calibration';
  import { StepChip } from '../../../ui/step-chips';
  import { channelNumbers } from './channels';

  const { rows }: { rows: NonNullable<CalibrationDialogModel['table']> } = $props();

  const shown = (value: { text: string; unit: string }) => `${value.text} ${value.unit}`;
</script>

<!-- A scrollable region must take the keyboard focus to scroll with the arrow keys. -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div class="scroll" role="region" aria-label="Stored offsets per voltage range and channel, scrollable" tabindex="0">
  <table>
    <thead>
      <tr>
        <th scope="col">Range</th>
        {#each channelNumbers as channel (channel)}
          <th scope="col">
            <StepChip
              label={String(channel)}
              state="done"
              check={false}
              compact
              color="var(--ss-color-ch{channel})"
              name="Channel {channel}"
            />
          </th>
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each rows as row (row.range.text + row.range.unit)}
        <tr>
          <th scope="row">{shown(row.range)}</th>
          {#each row.offsets as offset, index (index)}
            <td>
              {#if offset}{shown(offset)}{:else}<span class="none">-</span>{/if}
            </td>
          {/each}
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<style>
  .scroll {
    max-block-size: 11.5rem;
    overflow: auto;
    border: var(--ss-border-width) solid var(--ss-color-border);
  }
  .scroll:focus-visible {
    outline: var(--ss-border-width-focus) solid var(--ss-color-focus);
    outline-offset: var(--ss-border-width);
  }
  table {
    inline-size: 100%;
    border-collapse: collapse;
    font-size: var(--ss-font-size-sm);
  }
  thead th {
    position: sticky;
    inset-block-start: 0;
    z-index: 1;
    background: var(--ss-color-bg-raised);
    padding: var(--ss-space-1) var(--ss-space-3);
    border-block-end: var(--ss-border-width) solid var(--ss-color-border);
    font-size: var(--ss-font-size-2xs);
    text-transform: uppercase;
    letter-spacing: var(--ss-letter-spacing-caps);
    color: var(--ss-color-text-muted);
    text-align: end;
  }
  thead th:first-child,
  tbody th {
    text-align: start;
  }
  tbody th {
    font-weight: var(--ss-font-weight-regular);
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-secondary);
  }
  tbody th,
  td {
    padding: var(--ss-space-1) var(--ss-space-3);
    border-block-end: var(--ss-border-width) solid var(--ss-color-border-subtle);
  }
  td {
    text-align: end;
    font-family: var(--ss-font-mono);
    color: var(--ss-color-text-primary);
  }
  .none {
    color: var(--ss-color-text-muted);
  }
</style>
